//! Path sanitization and privacy redaction utilities for DevContext exports.
//!
//! Enforces ADR 0009:
//! - Absolute filesystem paths within the workspace are converted to workspace-relative paths.
//! - External paths have machine-specific user directory prefixes redacted to avoid leaking usernames.
//! - With explicit `--redact-paths`, external paths are stripped to generic placeholders.

use std::path::{Component, Path};

/// Sanitizes a reference target or filesystem path string relative to `workspace_root`.
pub fn sanitize_target(target: &str, workspace_root: &Path, aggressive_redaction: bool) -> String {
    let trimmed = target.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // Split anchor (e.g. #L10-L20) if present
    let (path_part, anchor_part) = match trimmed.split_once('#') {
        Some((p, a)) => (p, Some(a)),
        None => (trimmed, None),
    };

    let norm_path = normalize_slashes(path_part);
    let norm_root = normalize_slashes(&workspace_root.to_string_lossy());

    // If it's not an absolute path, normalize forward slashes and return
    if !is_absolute_path_str(&norm_path) {
        return match anchor_part {
            Some(a) => format!("{}#{}", norm_path, a),
            None => norm_path,
        };
    }

    // Try relativizing against workspace root
    let root_prefix = norm_root.trim_end_matches('/');
    if let Some(remainder) = norm_path.strip_prefix(root_prefix) {
        let remainder = remainder.trim_start_matches('/');
        let prefixed = if remainder.is_empty() {
            ".".to_string()
        } else {
            format!("./{}", remainder)
        };
        return match anchor_part {
            Some(a) => format!("{}#{}", prefixed, a),
            None => prefixed,
        };
    }

    // External path: apply redaction
    let sanitized_external = if aggressive_redaction {
        if let Some(pos) = norm_path.rfind('/') {
            let filename = &norm_path[pos + 1..];
            format!("<redacted>/{}", filename)
        } else {
            "<redacted>".to_string()
        }
    } else {
        redact_machine_specific_prefix(&norm_path)
    };

    match anchor_part {
        Some(a) => format!("{}#{}", sanitized_external, a),
        None => sanitized_external,
    }
}

/// Checks whether a path string represents an absolute path on Unix or Windows.
pub fn is_absolute_path_str(p: &str) -> bool {
    let norm = normalize_slashes(p);
    norm.starts_with('/')
        || norm.starts_with('\\')
        || (norm.len() >= 2 && norm.as_bytes()[1] == b':')
}

/// Redacts user directory prefixes like `/home/username/` or `C:/Users/username/` while preserving subpaths.
pub fn redact_machine_specific_prefix(path_str: &str) -> String {
    let norm = normalize_slashes(path_str);
    let segments: Vec<&str> = norm.split('/').filter(|s| !s.is_empty()).collect();

    for i in 0..segments.len() {
        if segments[i].eq_ignore_ascii_case("users") || segments[i].eq_ignore_ascii_case("home") {
            // Check if there is a username component after it
            if i + 2 < segments.len() {
                let remainder = segments[i + 2..].join("/");
                return format!("<redacted>/{}", remainder);
            } else {
                return "<redacted>".to_string();
            }
        }
    }

    // If no recognized user directory prefix, but path is absolute, replace root prefix with filename
    if is_absolute_path_str(&norm) {
        if let Some(pos) = norm.rfind('/') {
            let filename = &norm[pos + 1..];
            return format!("<redacted>/{}", filename);
        }
        return "<redacted>".to_string();
    }

    norm
}

/// Normalizes backslashes to forward slashes for cross-platform stability.
pub fn normalize_slashes(s: &str) -> String {
    s.replace('\\', "/")
}

/// Validates that a relative path extracted from an archive does not escape the destination directory
/// and contains no malicious path components (Zip Slip, Alternate Data Streams, null bytes, UNC paths).
pub fn is_safe_archive_path(entry_path: &Path) -> bool {
    let path_str = entry_path.to_string_lossy();

    // 1. Rejection of null bytes
    if path_str.contains('\0') {
        return false;
    }

    // 2. Rejection of Windows Alternate Data Streams and drive prefixes (colons)
    if path_str.contains(':') {
        return false;
    }

    let norm = normalize_slashes(&path_str);
    let trimmed = norm.trim();

    // 3. Rejection of absolute POSIX and UNC network paths
    if is_absolute_path_str(trimmed) || trimmed.starts_with("//") {
        return false;
    }

    // 4. Segment-by-segment inspection
    for segment in trimmed.split('/') {
        let seg = segment.trim();
        if seg == ".." {
            return false;
        }
        // Disallow leading/trailing whitespace tricks in filenames on Windows
        if seg.starts_with(' ') || seg.ends_with(' ') {
            return false;
        }
    }

    // 5. Rust path components inspection
    for component in entry_path.components() {
        match component {
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return false,
            Component::Normal(_) | Component::CurDir => continue,
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relative_path_normalization() {
        let root = Path::new("/workspace/project");
        let sanitized = sanitize_target("src/main.rs#L10-L20", root, false);
        assert_eq!(sanitized, "src/main.rs#L10-L20");
    }

    #[test]
    fn test_internal_absolute_path_relativization() {
        let root = Path::new("/workspace/project");
        let internal = "/workspace/project/src/controllers/user.rs";
        let sanitized = sanitize_target(internal, root, false);
        assert_eq!(sanitized, "./src/controllers/user.rs");
    }

    #[test]
    fn test_external_path_redaction() {
        let root = Path::new("/workspace/project");
        let external = "/home/john_doe/secret/credentials.txt";
        let sanitized = sanitize_target(external, root, false);
        assert_eq!(sanitized, "<redacted>/secret/credentials.txt");

        let aggressive = sanitize_target(external, root, true);
        assert_eq!(aggressive, "<redacted>/credentials.txt");
    }

    #[test]
    fn test_archive_path_containment() {
        assert!(is_safe_archive_path(Path::new("notes/test.md")));
        assert!(is_safe_archive_path(Path::new("export-v1.json")));
        assert!(!is_safe_archive_path(Path::new("../evil.txt")));
        assert!(!is_safe_archive_path(Path::new("notes/../../evil.txt")));
        assert!(!is_safe_archive_path(Path::new("/etc/passwd")));
        assert!(!is_safe_archive_path(Path::new("C:\\Windows\\System32")));
    }
}
