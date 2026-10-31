//! ZIP packaging and traversal-safe extraction for DevContext portable bundles.

use crate::error::{CoreError, Result};
use crate::export::markdown::generate_markdown_export;
use crate::export::model::{
    ExportBundle, MAX_ARCHIVE_ENTRIES, MAX_ARCHIVE_SIZE_BYTES, MAX_COMPRESSION_RATIO,
    MAX_ENTRY_UNCOMPRESSED_BYTES, MAX_JSON_PAYLOAD_BYTES, MAX_TOTAL_UNCOMPRESSED_BYTES,
};
use crate::export::sanitization::{is_safe_archive_path, normalize_slashes};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

/// Creates a standard `.zip` archive containing `export-v1.json` and human-readable Markdown files.
pub fn create_zip_archive(bundle: &ExportBundle, output_path: &Path) -> Result<()> {
    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    let temp_dir = tempfile::tempdir()?;
    let temp_root = temp_dir.path();

    // 1. Write root export-v1.json
    let json_content = serde_json::to_string_pretty(bundle)
        .map_err(|e| CoreError::Validation(format!("Failed to serialize bundle: {}", e)))?;
    fs::write(temp_root.join("export-v1.json"), json_content)?;

    // 2. Generate Markdown files
    generate_markdown_export(bundle, temp_root)?;

    // 3. Compress all files into ZIP
    let file = File::create(output_path)?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    add_dir_to_zip(&mut zip, temp_root, temp_root, options)?;
    zip.finish()?;

    Ok(())
}

fn add_dir_to_zip<W: Write + std::io::Seek>(
    zip: &mut ZipWriter<W>,
    base_dir: &Path,
    current_dir: &Path,
    options: SimpleFileOptions,
) -> Result<()> {
    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();
        let rel_path = path
            .strip_prefix(base_dir)
            .map_err(|e| CoreError::Io(std::io::Error::other(e)))?;
        let rel_str = normalize_slashes(&rel_path.to_string_lossy());

        if path.is_dir() {
            let dir_options = options.unix_permissions(0o755);
            zip.add_directory(format!("{}/", rel_str), dir_options)?;
            add_dir_to_zip(zip, base_dir, &path, options)?;
        } else if path.is_file() {
            zip.start_file(rel_str, options)?;
            let mut f = File::open(&path)?;
            std::io::copy(&mut f, zip)?;
        }
    }
    Ok(())
}

/// Safely extracts a ZIP archive into `dest_dir` with strict path traversal checks,
/// entry-count limits, and decompression-ratio / zip-bomb guards.
pub fn extract_zip_safely(zip_path: &Path, dest_dir: &Path) -> Result<PathBuf> {
    if !zip_path.exists() {
        return Err(CoreError::NotFound(format!(
            "Archive file not found: '{}'",
            zip_path.display()
        )));
    }

    let file = File::open(zip_path)?;
    let metadata = file.metadata()?;
    if metadata.len() > MAX_ARCHIVE_SIZE_BYTES {
        return Err(CoreError::Validation(format!(
            "Archive file size ({} bytes) exceeds maximum permitted limit ({} MB)",
            metadata.len(),
            MAX_ARCHIVE_SIZE_BYTES / 1024 / 1024
        )));
    }

    fs::create_dir_all(dest_dir)?;
    let canonical_dest = dest_dir.canonicalize()?;

    let mut archive = ZipArchive::new(file)?;
    if archive.len() > MAX_ARCHIVE_ENTRIES {
        let _ = fs::remove_dir_all(dest_dir);
        return Err(CoreError::Validation(format!(
            "Archive contains {} entries, exceeding maximum allowed limit ({})",
            archive.len(),
            MAX_ARCHIVE_ENTRIES
        )));
    }

    let mut total_uncompressed_bytes: u64 = 0;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| {
            let _ = fs::remove_dir_all(dest_dir);
            CoreError::Validation(format!(
                "Corrupted or unreadable archive entry at index {}: {}",
                i, e
            ))
        })?;

        let raw_name = entry
            .name()
            .map_err(|e| {
                let _ = fs::remove_dir_all(dest_dir);
                CoreError::Validation(format!(
                    "Invalid archive entry filename at index {}: {}",
                    i, e
                ))
            })?
            .to_string();
        let entry_path = PathBuf::from(&raw_name);

        // Security Check 1: Reject entries that are not safe relative paths
        if !is_safe_archive_path(&entry_path) {
            let _ = fs::remove_dir_all(dest_dir);
            return Err(CoreError::Validation(format!(
                "Path traversal security violation in archive entry: '{}'",
                raw_name
            )));
        }

        // Security Check 2: Reject symbolic links to prevent sandbox escape
        if let Some(mode) = entry.unix_mode() {
            if (mode & 0o170000) == 0o120000 {
                let _ = fs::remove_dir_all(dest_dir);
                return Err(CoreError::Validation(format!(
                    "Archive contains forbidden symbolic link entry: '{}'",
                    raw_name
                )));
            }
        }

        // Security Check 3: Resource & Zip-bomb limit verification
        let entry_uncompressed = entry.size();
        if entry_uncompressed > MAX_ENTRY_UNCOMPRESSED_BYTES {
            let _ = fs::remove_dir_all(dest_dir);
            return Err(CoreError::Validation(format!(
                "Archive entry '{}' uncompressed size exceeds limit ({} MB)",
                raw_name,
                MAX_ENTRY_UNCOMPRESSED_BYTES / 1024 / 1024
            )));
        }

        let entry_compressed = entry.compressed_size();
        if entry_compressed > 0
            && entry_uncompressed / entry_compressed > MAX_COMPRESSION_RATIO
            && entry_uncompressed > 1024 * 1024
        {
            let _ = fs::remove_dir_all(dest_dir);
            return Err(CoreError::Validation(format!(
                "Suspicious compression ratio (zip-bomb protection) in entry '{}'",
                raw_name
            )));
        }

        let outpath = dest_dir.join(&entry_path);

        // Security Check 4: Verify normalized target stays strictly within canonical destination
        if let Some(parent) = outpath.parent() {
            fs::create_dir_all(parent)?;
            let canonical_parent = parent.canonicalize()?;
            if !canonical_parent.starts_with(&canonical_dest) {
                let _ = fs::remove_dir_all(dest_dir);
                return Err(CoreError::Validation(format!(
                    "Archive entry escapes destination directory: '{}'",
                    raw_name
                )));
            }
        }

        if entry.is_dir() {
            fs::create_dir_all(&outpath)?;
        } else {
            let mut outfile = File::create(&outpath).inspect_err(|_| {
                let _ = fs::remove_dir_all(dest_dir);
            })?;
            let mut limited = (&mut entry).take(MAX_ENTRY_UNCOMPRESSED_BYTES + 1);
            let written = std::io::copy(&mut limited, &mut outfile).inspect_err(|_| {
                let _ = fs::remove_dir_all(dest_dir);
            })?;

            if written > MAX_ENTRY_UNCOMPRESSED_BYTES {
                let _ = fs::remove_dir_all(dest_dir);
                return Err(CoreError::Validation(format!(
                    "Decompressed data for entry '{}' exceeded maximum allowed size",
                    raw_name
                )));
            }

            total_uncompressed_bytes += written;
            if total_uncompressed_bytes > MAX_TOTAL_UNCOMPRESSED_BYTES {
                let _ = fs::remove_dir_all(dest_dir);
                return Err(CoreError::Validation(format!(
                    "Total uncompressed archive size exceeds maximum limit ({} MB)",
                    MAX_TOTAL_UNCOMPRESSED_BYTES / 1024 / 1024
                )));
            }
        }
    }

    Ok(canonical_dest)
}

/// Reads and parses `export-v1.json` directly from a ZIP file without full extraction.
pub fn read_bundle_from_zip(zip_path: &Path) -> Result<ExportBundle> {
    if !zip_path.exists() {
        return Err(CoreError::NotFound(format!(
            "Archive file not found: '{}'",
            zip_path.display()
        )));
    }

    let file = File::open(zip_path)?;
    let metadata = file.metadata()?;
    if metadata.len() > MAX_ARCHIVE_SIZE_BYTES {
        return Err(CoreError::Validation(format!(
            "Archive file size exceeds limit ({} MB)",
            MAX_ARCHIVE_SIZE_BYTES / 1024 / 1024
        )));
    }

    let mut archive = ZipArchive::new(file)?;
    if archive.len() > MAX_ARCHIVE_ENTRIES {
        return Err(CoreError::Validation(format!(
            "Archive contains {} entries, exceeding maximum allowed limit ({})",
            archive.len(),
            MAX_ARCHIVE_ENTRIES
        )));
    }

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| {
            CoreError::Validation(format!("Corrupted archive entry at index {}: {}", i, e))
        })?;
        let name = entry
            .name()
            .map_err(|e| {
                CoreError::Validation(format!(
                    "Invalid archive entry filename at index {}: {}",
                    i, e
                ))
            })?
            .to_string();
        let normalized = normalize_slashes(&name);

        if normalized == "export-v1.json" || normalized.ends_with("/export-v1.json") {
            if entry.size() > MAX_JSON_PAYLOAD_BYTES {
                return Err(CoreError::Validation(format!(
                    "export-v1.json payload size exceeds maximum limit ({} MB)",
                    MAX_JSON_PAYLOAD_BYTES / 1024 / 1024
                )));
            }

            let mut contents = String::new();
            let mut limited = (&mut entry).take(MAX_JSON_PAYLOAD_BYTES + 1);
            limited.read_to_string(&mut contents)?;

            if contents.len() as u64 > MAX_JSON_PAYLOAD_BYTES {
                return Err(CoreError::Validation(
                    "export-v1.json exceeds maximum payload limit".to_string(),
                ));
            }

            let raw: serde_json::Value = serde_json::from_str(&contents).map_err(|e| {
                CoreError::Validation(format!("Invalid export-v1.json in archive: {}", e))
            })?;
            let bundle = crate::export::service::ExportService::upgrade_bundle(raw)?;
            return Ok(bundle);
        }
    }

    Err(CoreError::NotFound(
        "Archive does not contain a valid 'export-v1.json' root manifest.".to_string(),
    ))
}
