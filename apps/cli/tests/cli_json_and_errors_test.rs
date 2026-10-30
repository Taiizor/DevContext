#![allow(clippy::needless_borrows_for_generic_args)]

use devcontext_cli::run_from_iter;
use tempfile::tempdir;

#[test]
fn test_cli_json_envelope_and_error_reporting() {
    let dir = tempdir().unwrap();
    let root_str = dir.path().to_str().unwrap();

    // 1. Uninitialized project status
    assert_eq!(
        run_from_iter(&["devcontext", "--json", "--project-dir", root_str, "status"]),
        1
    );

    // 2. Initialize with JSON
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--json",
            "--project-dir",
            root_str,
            "init",
            "--name",
            "JsonApp"
        ]),
        0
    );

    // 3. Add note with JSON
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--json",
            "--project-dir",
            root_str,
            "note",
            "add",
            "JSON Envelope Note",
            "--content",
            "Testing envelope format",
            "--tag",
            "json"
        ]),
        0
    );

    // 4. List notes with JSON
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--json",
            "--project-dir",
            root_str,
            "note",
            "list"
        ]),
        0
    );

    // 5. Show note with invalid UUID
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--json",
            "--project-dir",
            root_str,
            "note",
            "show",
            "not-a-valid-uuid"
        ]),
        1
    );

    // 6. Show non-existent UUID
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--json",
            "--project-dir",
            root_str,
            "note",
            "show",
            "00000000-0000-0000-0000-000000000001"
        ]),
        1
    );
}

#[test]
fn test_cli_decision_validation_errors() {
    let dir = tempdir().unwrap();
    let root_str = dir.path().to_str().unwrap();
    assert_eq!(
        run_from_iter(&["devcontext", "--project-dir", root_str, "init"]),
        0
    );

    // Invalid status (not in enum)
    let exit_invalid_status = run_from_iter(&[
        "devcontext",
        "--project-dir",
        root_str,
        "decision",
        "add",
        "Invalid Status Decision",
        "--context",
        "Context",
        "--decision",
        "Decision",
        "--consequences",
        "Consequences",
        "--status",
        "not_a_valid_status",
    ]);
    assert_eq!(exit_invalid_status, 1);
}
