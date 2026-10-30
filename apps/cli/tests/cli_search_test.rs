#![allow(clippy::needless_borrows_for_generic_args)]

use devcontext_cli::run_from_iter;
use tempfile::tempdir;

#[test]
fn test_cli_search_command_workflows() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let root_str = root.to_str().unwrap();

    // 1. In an uninitialized directory, search fails with NOT_FOUND and does not auto-init
    let exit_code = run_from_iter(&[
        "devcontext",
        "--project-dir",
        root_str,
        "search",
        "authentication",
    ]);
    assert_eq!(exit_code, 1);
    assert!(!root.join(".devcontext").exists());

    // 2. Initialize project
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "init",
            "--name",
            "SearchApp"
        ]),
        0
    );

    // 3. Add note
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "note",
            "add",
            "Database Migration Guide",
            "--content",
            "Detailed instructions on SQLite WAL mode and FTS5 schema migrations.",
            "--tag",
            "database",
            "--tag",
            "sqlite"
        ]),
        0
    );

    // 4. Add ADR
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "decision",
            "add",
            "ADR 0008: Storage Architecture",
            "--context",
            "We need a reliable local storage engine.",
            "--decision",
            "Adopt SQLite with WAL and FTS5 full-text indexing.",
            "--consequences",
            "Fast queries and zero remote services."
        ]),
        0
    );

    // 5. Basic search human output
    assert_eq!(
        run_from_iter(&["devcontext", "--project-dir", root_str, "search", "SQLite"]),
        0
    );

    // 6. Search with --type filter
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "search",
            "SQLite",
            "--type",
            "decision"
        ]),
        0
    );

    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "search",
            "SQLite",
            "--type",
            "note"
        ]),
        0
    );

    // 7. Search with --limit
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "search",
            "SQLite",
            "--limit",
            "1"
        ]),
        0
    );

    // 8. Search in JSON mode
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--json",
            "--project-dir",
            root_str,
            "search",
            "FTS5"
        ]),
        0
    );

    // 9. Search with no matches
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "search",
            "nonexistent_term_xyz_12345"
        ]),
        0
    );

    // 10. Search with invalid record type returns validation error code 1
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "search",
            "test",
            "--type",
            "invalid_type"
        ]),
        1
    );

    // 11. Search index rebuild via --reindex
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "search",
            "--reindex"
        ]),
        0
    );

    // 12. Search index rebuild in JSON mode
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--json",
            "--project-dir",
            root_str,
            "search",
            "--reindex"
        ]),
        0
    );
}
