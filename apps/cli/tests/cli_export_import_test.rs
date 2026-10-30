#![allow(clippy::needless_borrows_for_generic_args)]

use devcontext_cli::run_from_iter;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_cli_export_import_and_inspect_workflows() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let root_str = root.to_str().unwrap();

    // 1. Initialize project and add records
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "init",
            "--name",
            "ExportCliTest"
        ]),
        0
    );

    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "note",
            "add",
            "Cli Export Note",
            "--content",
            "Content to be exported and inspected.",
            "--tag",
            "export",
        ]),
        0
    );

    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "decision",
            "add",
            "ADR-Cli: Export Feature",
            "--status",
            "accepted",
            "--context",
            "Portability requirements.",
            "--decision",
            "Use standard JSON and ZIP formats.",
            "--consequences",
            "Easy interchange across environments.",
        ]),
        0
    );

    // 2. Export Preview
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "export",
            "--preview"
        ]),
        0
    );
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "--json",
            "export",
            "--preview"
        ]),
        0
    );

    // 3. Export to JSON
    let export_json = root.join("exported.json");
    let export_json_str = export_json.to_str().unwrap();
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "export",
            "--format",
            "json",
            "--output",
            export_json_str,
        ]),
        0
    );
    assert!(export_json.exists());

    // 4. Export to ZIP archive
    let export_zip = root.join("exported.zip");
    let export_zip_str = export_zip.to_str().unwrap();
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "export",
            "--format",
            "zip",
            "--output",
            export_zip_str,
            "--redact-paths",
        ]),
        0
    );
    assert!(export_zip.exists());

    // 5. Inspect database and inspect archive
    assert_eq!(
        run_from_iter(&["devcontext", "--project-dir", root_str, "inspect"]),
        0
    );
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "--json",
            "inspect",
            export_zip_str,
        ]),
        0
    );

    // 6. Import into a clean second project
    let dir2 = tempdir().unwrap();
    let root2 = dir2.path();
    let root2_str = root2.to_str().unwrap();

    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root2_str,
            "init",
            "--name",
            "ImportTarget"
        ]),
        0
    );

    // Import with dry-run
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root2_str,
            "import",
            export_json_str,
            "--dry-run",
        ]),
        0
    );

    // Real import from ZIP
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root2_str,
            "import",
            export_zip_str,
            "--on-conflict",
            "skip",
        ]),
        0
    );

    // Verify imported records in project 2 via status
    assert_eq!(
        run_from_iter(&["devcontext", "--project-dir", root2_str, "status"]),
        0
    );

    // 7. Delete data without --force fails
    assert_eq!(
        run_from_iter(&["devcontext", "--project-dir", root2_str, "delete-data"]),
        1
    );

    // Delete data with --force succeeds
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root2_str,
            "delete-data",
            "--force",
        ]),
        0
    );

    // Verify backup exists in .devcontext/backups/
    let backups_dir = root2.join(".devcontext").join("backups");
    assert!(backups_dir.is_dir());
    let backup_entries = fs::read_dir(backups_dir).unwrap().count();
    assert!(backup_entries >= 1);
}
