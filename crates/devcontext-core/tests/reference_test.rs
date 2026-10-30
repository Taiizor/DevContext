use devcontext_core::domain::{Note, Project, Reference, ReferenceType};
use devcontext_core::reference::{ReferenceResolutionStatus, ReferenceResolver};
use devcontext_core::storage::SqliteStorage;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;
use uuid::Uuid;

fn init_git_repo(path: &Path) {
    Command::new("git")
        .arg("init")
        .current_dir(path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["config", "user.name", "Reference Tester"])
        .current_dir(path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["config", "user.email", "ref@devcontext.local"])
        .current_dir(path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["config", "commit.gpgsign", "false"])
        .current_dir(path)
        .output()
        .unwrap();
}

fn create_commit(path: &Path, file_name: &str, content: &str) -> String {
    let file_path = path.join(file_name);
    let mut file = File::create(&file_path).unwrap();
    file.write_all(content.as_bytes()).unwrap();

    Command::new("git")
        .args(["add", file_name])
        .current_dir(path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["commit", "-m", "add test file"])
        .current_dir(path)
        .output()
        .unwrap();

    let out = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(path)
        .output()
        .unwrap();

    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn test_git_commit_reference_resolution() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let rec_id = Uuid::new_v4();

    // 1. In a non-git directory, status is Unavailable
    let ref_unavail = Reference::new(
        rec_id,
        ReferenceType::GitCommit,
        "a1b2c3d4e5f6a1b2c3d4",
        None,
        None,
    )
    .unwrap();
    let res_unavail = ReferenceResolver::resolve(&ref_unavail, root, None);
    assert_eq!(res_unavail.status, ReferenceResolutionStatus::Unavailable);

    // 2. Malformed commit hash is Invalid
    let ref_malformed = Reference::new(
        rec_id,
        ReferenceType::GitCommit,
        "not-a-hex-hash!",
        None,
        None,
    )
    .unwrap();
    let res_malformed = ReferenceResolver::resolve(&ref_malformed, root, None);
    assert_eq!(res_malformed.status, ReferenceResolutionStatus::Invalid);

    // 3. Initialize Git repository
    init_git_repo(root);
    let commit_hash = create_commit(root, "file.txt", "v1");

    // Existing commit is Resolved
    let ref_valid =
        Reference::new(rec_id, ReferenceType::GitCommit, &commit_hash, None, None).unwrap();
    let res_valid = ReferenceResolver::resolve(&ref_valid, root, None);
    assert_eq!(res_valid.status, ReferenceResolutionStatus::Resolved);

    // Non-existent commit hash is TargetMissing
    let ref_missing = Reference::new(
        rec_id,
        ReferenceType::GitCommit,
        "0000000000000000000000000000000000000000",
        None,
        None,
    )
    .unwrap();
    let res_missing = ReferenceResolver::resolve(&ref_missing, root, None);
    assert_eq!(res_missing.status, ReferenceResolutionStatus::TargetMissing);
}

#[test]
fn test_file_and_folder_reference_resolution_and_path_traversal() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let rec_id = Uuid::new_v4();

    // Create a real file and folder in the workspace
    let src_dir = root.join("src").join("models");
    fs::create_dir_all(&src_dir).unwrap();
    let test_file = src_dir.join("user.rs");
    fs::write(&test_file, "pub struct User;").unwrap();

    // 1. Existing file resolves
    let ref_file = Reference::new(
        rec_id,
        ReferenceType::File,
        "src/models/user.rs",
        None,
        None,
    )
    .unwrap();
    let res_file = ReferenceResolver::resolve(&ref_file, root, None);
    assert_eq!(res_file.status, ReferenceResolutionStatus::Resolved);

    // 2. Missing file returns TargetMissing
    let ref_missing = Reference::new(
        rec_id,
        ReferenceType::File,
        "src/models/missing.rs",
        None,
        None,
    )
    .unwrap();
    let res_missing = ReferenceResolver::resolve(&ref_missing, root, None);
    assert_eq!(res_missing.status, ReferenceResolutionStatus::TargetMissing);

    // 3. Pointing to a folder when expecting a file returns Invalid
    let ref_dir_as_file =
        Reference::new(rec_id, ReferenceType::File, "src/models", None, None).unwrap();
    let res_dir_as_file = ReferenceResolver::resolve(&ref_dir_as_file, root, None);
    assert_eq!(res_dir_as_file.status, ReferenceResolutionStatus::Invalid);

    // 4. Existing folder resolves
    let ref_folder =
        Reference::new(rec_id, ReferenceType::Folder, "src/models", None, None).unwrap();
    let res_folder = ReferenceResolver::resolve(&ref_folder, root, None);
    assert_eq!(res_folder.status, ReferenceResolutionStatus::Resolved);

    // 5. Security: Path traversal attempts (..) escaping project root must be Invalid
    let ref_traversal_1 =
        Reference::new(rec_id, ReferenceType::File, "../outside.txt", None, None).unwrap();
    let res_traversal_1 = ReferenceResolver::resolve(&ref_traversal_1, root, None);
    assert_eq!(res_traversal_1.status, ReferenceResolutionStatus::Invalid);

    let ref_traversal_2 = Reference::new(
        rec_id,
        ReferenceType::File,
        "src/../../outside.txt",
        None,
        None,
    )
    .unwrap();
    let res_traversal_2 = ReferenceResolver::resolve(&ref_traversal_2, root, None);
    assert_eq!(res_traversal_2.status, ReferenceResolutionStatus::Invalid);
}

#[test]
fn test_reference_path_traversal_and_symlink_security() {
    let project_dir = tempdir().unwrap();
    let root = project_dir.path();
    let rec_id = Uuid::new_v4();

    // Create an outside directory completely separate from project
    let outside_dir = tempdir().unwrap();
    let outside_root = outside_dir.path();
    let outside_file = outside_root.join("secret_data.txt");
    fs::write(&outside_file, "super-secret").unwrap();

    // 1. Absolute paths pointing outside project root
    let ref_abs = Reference::new(
        rec_id,
        ReferenceType::File,
        outside_file.to_str().unwrap(),
        None,
        None,
    )
    .unwrap();
    let res_abs = ReferenceResolver::resolve(&ref_abs, root, None);
    assert_eq!(res_abs.status, ReferenceResolutionStatus::Invalid);

    // 2. Nonexistent target in an external directory
    let ref_abs_missing = Reference::new(
        rec_id,
        ReferenceType::File,
        outside_root.join("nonexistent.txt").to_str().unwrap(),
        None,
        None,
    )
    .unwrap();
    let res_abs_missing = ReferenceResolver::resolve(&ref_abs_missing, root, None);
    assert_eq!(res_abs_missing.status, ReferenceResolutionStatus::Invalid);

    // 3. UNC and drive-letter traversal tests
    #[cfg(windows)]
    {
        let ref_unc = Reference::new(
            rec_id,
            ReferenceType::File,
            "\\\\127.0.0.1\\c$\\secret.txt",
            None,
            None,
        )
        .unwrap();
        let res_unc = ReferenceResolver::resolve(&ref_unc, root, None);
        assert_eq!(res_unc.status, ReferenceResolutionStatus::Invalid);
    }

    // 4. Windows NTFS Junctions / Symlinks pointing outside project root
    let junction_dir = root.join("external_junction");
    let mut junction_created = false;

    #[cfg(windows)]
    {
        // Try creating NTFS directory junction using Windows cmd mklink /J
        let status = Command::new("cmd")
            .args([
                "/C",
                "mklink",
                "/J",
                junction_dir.to_str().unwrap(),
                outside_root.to_str().unwrap(),
            ])
            .output();

        if let Ok(out) = status {
            if out.status.success() {
                junction_created = true;
            }
        }
    }

    #[cfg(not(windows))]
    {
        if std::os::unix::fs::symlink(outside_root, &junction_dir).is_ok() {
            junction_created = true;
        }
    }

    if junction_created {
        // A: Existing file accessed via junction must NOT resolve; must be Invalid
        let ref_via_junction = Reference::new(
            rec_id,
            ReferenceType::File,
            "external_junction/secret_data.txt",
            None,
            None,
        )
        .unwrap();
        let res_via_junction = ReferenceResolver::resolve(&ref_via_junction, root, None);
        assert_eq!(
            res_via_junction.status,
            ReferenceResolutionStatus::Invalid,
            "External symlink/junction access must be Invalid"
        );

        // B: Nonexistent file accessed via external junction must NOT be TargetMissing; must be Invalid
        let ref_missing_via_junction = Reference::new(
            rec_id,
            ReferenceType::File,
            "external_junction/not_present.txt",
            None,
            None,
        )
        .unwrap();
        let res_missing_via_junction =
            ReferenceResolver::resolve(&ref_missing_via_junction, root, None);
        assert_eq!(
            res_missing_via_junction.status,
            ReferenceResolutionStatus::Invalid,
            "Ancestor resolving outside root via junction must be Invalid"
        );
    }

    // 5. Internal symlink/junction pointing WITHIN project root is allowed
    let internal_target = root.join("internal_dir");
    fs::create_dir_all(&internal_target).unwrap();
    let internal_file = internal_target.join("internal_doc.md");
    fs::write(&internal_file, "# Document").unwrap();

    let internal_link = root.join("internal_link");
    let mut internal_link_created = false;

    #[cfg(windows)]
    {
        let status = Command::new("cmd")
            .args([
                "/C",
                "mklink",
                "/J",
                internal_link.to_str().unwrap(),
                internal_target.to_str().unwrap(),
            ])
            .output();
        if let Ok(out) = status {
            if out.status.success() {
                internal_link_created = true;
            }
        }
    }

    #[cfg(not(windows))]
    {
        if std::os::unix::fs::symlink(&internal_target, &internal_link).is_ok() {
            internal_link_created = true;
        }
    }

    if internal_link_created {
        // Existing internal target via link is Resolved
        let ref_internal = Reference::new(
            rec_id,
            ReferenceType::File,
            "internal_link/internal_doc.md",
            None,
            None,
        )
        .unwrap();
        let res_internal = ReferenceResolver::resolve(&ref_internal, root, None);
        assert_eq!(res_internal.status, ReferenceResolutionStatus::Resolved);

        // Missing internal target via link is TargetMissing
        let ref_internal_missing = Reference::new(
            rec_id,
            ReferenceType::File,
            "internal_link/missing.md",
            None,
            None,
        )
        .unwrap();
        let res_internal_missing = ReferenceResolver::resolve(&ref_internal_missing, root, None);
        assert_eq!(
            res_internal_missing.status,
            ReferenceResolutionStatus::TargetMissing
        );
    }
}

#[test]
fn test_url_reference_validation_without_network() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let rec_id = Uuid::new_v4();

    // Valid URLs
    let valid_urls = [
        "https://example.com/docs/api",
        "http://localhost:8080/metrics",
        "https://github.com/Taiizor/DevContext/pull/42",
        "git://github.com/repo.git",
    ];

    for u in valid_urls {
        let r = Reference::new(rec_id, ReferenceType::Url, u, None, None).unwrap();
        let res = ReferenceResolver::resolve(&r, root, None);
        assert_eq!(
            res.status,
            ReferenceResolutionStatus::Resolved,
            "URL {} should resolve",
            u
        );
    }

    // Malformed URLs
    let invalid_urls = [
        "not-a-url",
        "http://",
        "https://   ",
        "ftp://",
        "javascript:alert(1)",
        "https://example .com",
    ];

    for u in invalid_urls {
        let r = Reference::new(rec_id, ReferenceType::Url, u, None, None).unwrap();
        let res = ReferenceResolver::resolve(&r, root, None);
        assert_eq!(
            res.status,
            ReferenceResolutionStatus::Invalid,
            "URL {} should be invalid",
            u
        );
    }
}

#[test]
fn test_symbol_and_command_references() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let rec_id = Uuid::new_v4();

    // Symbol reference
    let ref_sym = Reference::new(
        rec_id,
        ReferenceType::Symbol,
        "SqliteStorage::save_note",
        Some("Storage entrypoint".into()),
        None,
    )
    .unwrap();
    let res_sym = ReferenceResolver::resolve(&ref_sym, root, None);
    assert_eq!(res_sym.status, ReferenceResolutionStatus::Resolved);

    // Command reference (never executed, stored as text)
    let ref_cmd = Reference::new(
        rec_id,
        ReferenceType::Command,
        "cargo test --workspace",
        Some("Test runner".into()),
        None,
    )
    .unwrap();
    let res_cmd = ReferenceResolver::resolve(&ref_cmd, root, None);
    assert_eq!(res_cmd.status, ReferenceResolutionStatus::Resolved);
}

#[test]
fn test_record_reference_resolution_and_integrity() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let storage = SqliteStorage::open_in_memory().unwrap();

    let project = Project::new("Record Ref Project", None).unwrap();
    storage.save_project(&project).unwrap();

    let target_note = Note::new(project.id, "Target Note", "Content", vec![], None).unwrap();
    storage.save_note(&target_note).unwrap();

    let caller_note = Note::new(project.id, "Caller Note", "Content", vec![], None).unwrap();
    storage.save_note(&caller_note).unwrap();

    // 1. Reference pointing to existing active record
    let ref_existing = Reference::new(
        caller_note.record.id,
        ReferenceType::Record,
        target_note.record.id.to_string(),
        Some("Links to target".into()),
        None,
    )
    .unwrap();

    let res_existing = ReferenceResolver::resolve(&ref_existing, root, Some(&storage));
    assert_eq!(res_existing.status, ReferenceResolutionStatus::Resolved);

    // 2. Reference pointing to non-existent UUID
    let random_id = Uuid::new_v4();
    let ref_missing = Reference::new(
        caller_note.record.id,
        ReferenceType::Record,
        random_id.to_string(),
        None,
        None,
    )
    .unwrap();

    let res_missing = ReferenceResolver::resolve(&ref_missing, root, Some(&storage));
    assert_eq!(res_missing.status, ReferenceResolutionStatus::TargetMissing);

    // 3. Reference with invalid UUID target string
    let ref_invalid_uuid = Reference::new(
        caller_note.record.id,
        ReferenceType::Record,
        "not-a-valid-uuid",
        None,
        None,
    )
    .unwrap();

    let res_invalid = ReferenceResolver::resolve(&ref_invalid_uuid, root, Some(&storage));
    assert_eq!(res_invalid.status, ReferenceResolutionStatus::Invalid);

    // 4. Soft-deleted record target becomes TargetMissing
    storage.soft_delete_record(target_note.record.id).unwrap();
    let res_soft_deleted = ReferenceResolver::resolve(&ref_existing, root, Some(&storage));
    assert_eq!(
        res_soft_deleted.status,
        ReferenceResolutionStatus::TargetMissing
    );
}

#[test]
fn test_multiple_references_batch_resolution() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let rec_id = Uuid::new_v4();

    // Write a dummy file
    fs::write(root.join("README.md"), "# Hello").unwrap();

    let refs = vec![
        Reference::new(rec_id, ReferenceType::File, "README.md", None, None).unwrap(),
        Reference::new(rec_id, ReferenceType::File, "nonexistent.txt", None, None).unwrap(),
        Reference::new(
            rec_id,
            ReferenceType::Url,
            "https://devcontext.local",
            None,
            None,
        )
        .unwrap(),
        Reference::new(rec_id, ReferenceType::Url, "bad-url", None, None).unwrap(),
    ];

    let resolved = ReferenceResolver::resolve_all(&refs, root, None);
    assert_eq!(resolved.len(), 4);
    assert_eq!(resolved[0].status, ReferenceResolutionStatus::Resolved);
    assert_eq!(resolved[1].status, ReferenceResolutionStatus::TargetMissing);
    assert_eq!(resolved[2].status, ReferenceResolutionStatus::Resolved);
    assert_eq!(resolved[3].status, ReferenceResolutionStatus::Invalid);
}
