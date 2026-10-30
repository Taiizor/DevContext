//! Phase 9: Automated validation of the shipped sample project dataset.

use devcontext_core::export::{ConflictStrategy, ExportService, ImportOptions};
use devcontext_core::search::{SearchFilter, SearchService};
use devcontext_core::workspace::WorkspaceManager;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

#[test]
fn test_sample_project_dataset_inspection_and_import_lifecycle() {
    // 1. Locate the sample export file in the repository
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("Must resolve repo root");
    let sample_file = repo_root.join("examples/sample-project/aurora-sample-export.json");

    assert!(
        sample_file.exists(),
        "Sample export file must exist at '{}'",
        sample_file.display()
    );

    // 2. Initialize a clean temporary workspace
    let dir = tempdir().unwrap();
    let root = dir.path();
    let (project, storage) =
        WorkspaceManager::initialize_project(root, Some("TestImportTarget")).unwrap();

    // 3. Inspect the sample bundle without modifying the workspace
    let report = ExportService::inspect_source(root, Some(&sample_file))
        .expect("Sample bundle must inspect cleanly");

    assert_eq!(report.project_name, "Aurora Workflow Engine");
    assert_eq!(report.schema_version.as_deref(), Some("1.0.0"));
    assert_eq!(report.notes_count, 1);
    assert_eq!(report.decisions_count, 2);
    assert_eq!(report.milestones_count, 2);
    assert_eq!(report.logs_count, 1);
    assert_eq!(report.commands_count, 1);
    assert_eq!(report.total_records, 7);

    // 4. Import the sample bundle into the workspace
    let import_opts = ImportOptions {
        on_conflict: ConflictStrategy::GenerateNewId,
        create_backup: true,
        dry_run: false,
    };

    let import_result = ExportService::import_bundle(root, &sample_file, &import_opts)
        .expect("Importing sample bundle must succeed");

    assert_eq!(import_result.imported_count, 7);
    assert_eq!(import_result.conflicts_count, 0);

    // 5. Verify records are searchable via FTS5
    let filter = SearchFilter::default();

    let sqlite_matches =
        SearchService::search(&storage, project.id, root, "sqlite", &filter).unwrap();
    assert!(
        !sqlite_matches.is_empty(),
        "Must find ADR regarding SQLite persistence"
    );
    assert_eq!(
        sqlite_matches[0].title,
        "ADR-001: Adopt Embedded SQLite with WAL for Job Persistence"
    );

    let dispatcher_matches =
        SearchService::search(&storage, project.id, root, "dispatcher", &filter).unwrap();
    assert!(
        !dispatcher_matches.is_empty(),
        "Must find Note regarding Task Dispatch Loop"
    );

    let ring_buffer_matches =
        SearchService::search(&storage, project.id, root, "buffer", &filter).unwrap();
    assert!(
        !ring_buffer_matches.is_empty(),
        "Must find ADR-002 regarding Ring Buffer"
    );

    // 6. Delete all project memory and verify automatic pre-deletion backup snapshot
    let delete_result =
        ExportService::delete_project_data(root, true, true).expect("Must delete data cleanly");

    assert_eq!(delete_result.deleted_records_count, 7);
    let backup_path = delete_result
        .backup_path
        .expect("Backup path must be returned");
    assert!(
        Path::new(&backup_path).exists(),
        "Backup file must be preserved after data deletion"
    );

    // Verify storage has 0 records after deletion
    let notes_after = storage.list_notes(project.id, false).unwrap();
    assert_eq!(notes_after.len(), 0);
}
