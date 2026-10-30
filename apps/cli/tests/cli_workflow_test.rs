#![allow(clippy::needless_borrows_for_generic_args)]

use devcontext_cli::run_from_iter;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_cli_init_status_doctor_lifecycle() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let root_str = root.to_str().unwrap();

    // 1. Status before init must return exit code 1 (NOT_FOUND)
    let exit_status_pre = run_from_iter(&["devcontext", "--project-dir", root_str, "status"]);
    assert_eq!(exit_status_pre, 1);

    // 2. Doctor before init (detects no project, exits 0)
    let exit_doctor_pre = run_from_iter(&["devcontext", "--project-dir", root_str, "doctor"]);
    assert_eq!(exit_doctor_pre, 0);

    // 3. Init with custom name
    let exit_init = run_from_iter(&[
        "devcontext",
        "--project-dir",
        root_str,
        "init",
        "--name",
        "MyCliProject",
    ]);
    assert_eq!(exit_init, 0);

    // Verify .devcontext directory and db created
    assert!(root.join(".devcontext").join("devcontext.db").exists());

    // 4. Duplicate init must fail with exit code 1
    let exit_reinit = run_from_iter(&["devcontext", "--project-dir", root_str, "init"]);
    assert_eq!(exit_reinit, 1);

    // 5. Status after init must succeed
    let exit_status_post = run_from_iter(&["devcontext", "--project-dir", root_str, "status"]);
    assert_eq!(exit_status_post, 0);

    // 6. Doctor after init must report project health
    let exit_doctor_post = run_from_iter(&["devcontext", "--project-dir", root_str, "doctor"]);
    assert_eq!(exit_doctor_post, 0);
}

#[test]
fn test_cli_nested_directory_discovery() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let root_str = root.to_str().unwrap();

    // Initialize at root
    assert_eq!(
        run_from_iter(&["devcontext", "--project-dir", root_str, "init"]),
        0
    );

    // Create deep nested directory: root/packages/core/src
    let nested = root.join("packages").join("core").join("src");
    fs::create_dir_all(&nested).unwrap();
    let nested_str = nested.to_str().unwrap();

    // Add note specifying nested directory as project_dir (should auto-discover root)
    let exit_add = run_from_iter(&[
        "devcontext",
        "--project-dir",
        nested_str,
        "note",
        "add",
        "Nested Discovery Note",
        "--content",
        "Auto-discovered from nested folder",
    ]);
    assert_eq!(exit_add, 0);

    // List notes from nested directory
    let exit_list = run_from_iter(&["devcontext", "--project-dir", nested_str, "note", "list"]);
    assert_eq!(exit_list, 0);
}

#[test]
fn test_cli_note_crud_workflow() {
    let dir = tempdir().unwrap();
    let root_str = dir.path().to_str().unwrap();
    assert_eq!(
        run_from_iter(&["devcontext", "--project-dir", root_str, "init"]),
        0
    );

    // 1. Add Note
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "note",
            "add",
            "Initial Note",
            "--content",
            "Original content",
            "--category",
            "backend",
            "--tag",
            "api",
            "--tag",
            "rust"
        ]),
        0
    );

    // 2. Read notes
    let (_, storage) =
        devcontext_core::workspace::WorkspaceManager::open_project(dir.path()).unwrap();
    let projects = storage.list_projects().unwrap();
    let project_id = projects[0].id;
    let notes = storage.list_notes(project_id, true).unwrap();
    assert_eq!(notes.len(), 1);
    let note_id = notes[0].record.id.to_string();

    // 3. Show Note
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "note",
            "show",
            &note_id
        ]),
        0
    );

    // 4. Edit Note (update title, pin, archive)
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "note",
            "edit",
            &note_id,
            "--title",
            "Updated Note Title",
            "--pin",
            "--archive"
        ]),
        0
    );

    let updated = storage.get_note(notes[0].record.id).unwrap().unwrap();
    assert_eq!(updated.record.title, "Updated Note Title");
    assert!(updated.is_pinned);
    assert!(updated.is_archived);

    // 5. Delete Note (soft-delete)
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "note",
            "delete",
            &note_id
        ]),
        0
    );
    assert!(storage.get_note(notes[0].record.id).unwrap().is_none());

    // 6. Delete again must return error 1
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "note",
            "delete",
            &note_id
        ]),
        1
    );
}

#[test]
fn test_cli_decision_workflow_and_supersede() {
    let dir = tempdir().unwrap();
    let root_str = dir.path().to_str().unwrap();
    assert_eq!(
        run_from_iter(&["devcontext", "--project-dir", root_str, "init"]),
        0
    );

    // 1. Add ADR 1
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "decision",
            "add",
            "ADR 0001: SQLite",
            "--context",
            "We need local storage",
            "--decision",
            "Use SQLite with WAL",
            "--consequences",
            "ACID and high performance",
            "--tag",
            "storage"
        ]),
        0
    );

    // 2. Add ADR 2
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "decision",
            "add",
            "ADR 0002: Encryption",
            "--context",
            "Need encrypted notes",
            "--decision",
            "Use SQLCipher",
            "--consequences",
            "Overhead but secure"
        ]),
        0
    );

    let (_, storage) =
        devcontext_core::workspace::WorkspaceManager::open_project(dir.path()).unwrap();
    let projects = storage.list_projects().unwrap();
    let decisions = storage.list_decisions(projects[0].id).unwrap();
    assert_eq!(decisions.len(), 2);

    let d1_id = decisions[1].record.id.to_string(); // ADR 1
    let d2_id = decisions[0].record.id.to_string(); // ADR 2

    // 3. Supersede ADR 1 with ADR 2
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "decision",
            "update",
            &d1_id,
            "--superseded-by",
            &d2_id
        ]),
        0
    );

    let d1_updated = storage
        .get_decision(decisions[1].record.id)
        .unwrap()
        .unwrap();
    assert_eq!(
        d1_updated.status,
        devcontext_core::domain::DecisionStatus::Superseded
    );
    assert_eq!(d1_updated.superseded_by, Some(decisions[0].record.id));
}

#[test]
fn test_cli_milestone_and_log_commands() {
    let dir = tempdir().unwrap();
    let root_str = dir.path().to_str().unwrap();
    assert_eq!(
        run_from_iter(&["devcontext", "--project-dir", root_str, "init"]),
        0
    );

    // 1. Add Milestone
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "milestone",
            "add",
            "Alpha Release",
            "--status",
            "completed",
            "--description",
            "First usable CLI"
        ]),
        0
    );

    // 2. List Milestones
    assert_eq!(
        run_from_iter(&["devcontext", "--project-dir", root_str, "milestone", "list"]),
        0
    );

    // 3. Add Log Entry
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "log",
            "add",
            "CLI Refactoring",
            "--summary",
            "Refactored CLI commands to thin handlers",
            "--details",
            "Split into args, output, and command modules"
        ]),
        0
    );

    // 4. List Log Entries
    assert_eq!(
        run_from_iter(&["devcontext", "--project-dir", root_str, "log", "list"]),
        0
    );
}

#[test]
fn test_cli_unicode_and_special_characters() {
    let dir = tempdir().unwrap();
    let root_str = dir.path().to_str().unwrap();

    // Unicode project name (Turkish / German / Japanese)
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "init",
            "--name",
            "DevContext Geliştirme Projesi 🚀 日本語"
        ]),
        0
    );

    // Unicode note title and body
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "note",
            "add",
            "Mimarî Kararlar & Açıklamalar",
            "--content",
            "Veritabanı güvenliği için WAL kipi ve UTF-8 desteği test edilmiştir. 🎯",
            "--tag",
            "türkçe",
            "--tag",
            "mimarî"
        ]),
        0
    );

    let (_, storage) =
        devcontext_core::workspace::WorkspaceManager::open_project(dir.path()).unwrap();
    let projects = storage.list_projects().unwrap();
    assert_eq!(projects[0].name, "DevContext Geliştirme Projesi 🚀 日本語");

    let notes = storage.list_notes(projects[0].id, true).unwrap();
    assert_eq!(notes[0].record.title, "Mimarî Kararlar & Açıklamalar");
    assert!(notes[0].content_markdown.contains("WAL kipi ve UTF-8"));
}

#[test]
fn test_cli_independent_projects_isolation() {
    let dir_a = tempdir().unwrap();
    let dir_b = tempdir().unwrap();

    let root_a = dir_a.path().to_str().unwrap();
    let root_b = dir_b.path().to_str().unwrap();

    // Init project A & B
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_a,
            "init",
            "--name",
            "Project A"
        ]),
        0
    );
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_b,
            "init",
            "--name",
            "Project B"
        ]),
        0
    );

    // Add note to A
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_a,
            "note",
            "add",
            "Secret Note in A",
            "--content",
            "A only content"
        ]),
        0
    );

    // Verify Project B has 0 notes
    let (_, storage_b) =
        devcontext_core::workspace::WorkspaceManager::open_project(dir_b.path()).unwrap();
    let projects_b = storage_b.list_projects().unwrap();
    let notes_b = storage_b.list_notes(projects_b[0].id, true).unwrap();
    assert_eq!(notes_b.len(), 0);
}
