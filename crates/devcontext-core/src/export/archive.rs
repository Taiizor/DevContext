//! ZIP packaging and traversal-safe extraction for DevContext portable bundles.

use crate::error::{CoreError, Result};
use crate::export::markdown::generate_markdown_export;
use crate::export::model::ExportBundle;
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

/// Safely extracts a ZIP archive into `dest_dir` with strict path traversal checks.
pub fn extract_zip_safely(zip_path: &Path, dest_dir: &Path) -> Result<PathBuf> {
    if !zip_path.exists() {
        return Err(CoreError::NotFound(format!(
            "Archive file not found: '{}'",
            zip_path.display()
        )));
    }

    fs::create_dir_all(dest_dir)?;
    let canonical_dest = dest_dir.canonicalize()?;

    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let raw_name = entry.name().to_string();
        let entry_path = PathBuf::from(&raw_name);

        // Security Check 1: Reject entries that are not safe relative paths
        if !is_safe_archive_path(&entry_path) {
            return Err(CoreError::Validation(format!(
                "Path traversal security violation in archive entry: '{}'",
                raw_name
            )));
        }

        let outpath = dest_dir.join(&entry_path);

        // Security Check 2: Verify normalized target stays strictly within canonical destination
        if let Some(parent) = outpath.parent() {
            fs::create_dir_all(parent)?;
            let canonical_parent = parent.canonicalize()?;
            if !canonical_parent.starts_with(&canonical_dest) {
                return Err(CoreError::Validation(format!(
                    "Archive entry escapes destination directory: '{}'",
                    raw_name
                )));
            }
        }

        if entry.is_dir() {
            fs::create_dir_all(&outpath)?;
        } else {
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut entry, &mut outfile)?;
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
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();
        let normalized = normalize_slashes(&name);

        if normalized == "export-v1.json" || normalized.ends_with("/export-v1.json") {
            let mut contents = String::new();
            entry.read_to_string(&mut contents)?;
            let bundle: ExportBundle = serde_json::from_str(&contents).map_err(|e| {
                CoreError::Validation(format!("Invalid export-v1.json in archive: {}", e))
            })?;
            return Ok(bundle);
        }
    }

    Err(CoreError::NotFound(
        "Archive does not contain a valid 'export-v1.json' root manifest.".to_string(),
    ))
}
