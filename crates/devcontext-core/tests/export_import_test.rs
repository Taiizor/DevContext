//! Phase 7: Comprehensive Integration Tests for Export, Import, Portability, and Data Safety.

use devcontext_core::domain::{
    Decision, DecisionStatus, LogEntry, Milestone, MilestoneStatus, Note, Project, SavedCommand,
};
use devcontext_core::export::{
    extract_zip_safely, ConflictStrategy, ExportFormat, ExportOptions, ExportService, ImportOptions,
};
use devcontext_core::storage::backup::BackupService;
use devcontext_core::storage::SqliteStorage;
use devcontext_core::workspace::WorkspaceManager;
use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

fn setup_sample_project(root: &std::path::Path) -> (Project, SqliteStorage) {
    let (project, storage) =
        WorkspaceManager::initialize_project(root, Some("SampleProject")).unwrap();

    // 1. Add Note with Turkish characters and tags
    let note = Note::new(
        project.id,
        "Türkçe Not Başlığı 🚀",
        "İçerik detayı: Şeker, ağaç, ılık su, çağlayan ve öğüt.",
        vec!["türkçe".to_string(), "unicode".to_string()],
        Some("debugging".to_string()),
    )
    .unwrap();
    storage.save_note(&note).unwrap();

    // 2. Add Decision (ADR)
    let dec = Decision::new(
        project.id,
        "ADR-0013: SQLite WAL Storage",
        DecisionStatus::Accepted,
        "Local-first requirements require robust offline storage.",
        "Use SQLite with WAL mode enabled.",
        "High performance, safe concurrency.",
        Some("Considered plain JSON files, rejected due to concurrency hazards.".to_string()),
        vec!["architecture".to_string(), "storage".to_string()],
    )
    .unwrap();
    storage.save_decision(&dec).unwrap();

    // 3. Add Milestone
    let ms = Milestone::new(
        project.id,
        "Release v1.0",
        MilestoneStatus::Active,
        Some("2026-12-31T23:59:59Z".to_string()),
        Some("First public release with full CLI and VS Code support.".to_string()),
        vec!["release".to_string()],
    )
    .unwrap();
    storage.save_milestone(&ms).unwrap();

    // 4. Add Log Entry
    let log = LogEntry::new(
        project.id,
        "Migration Completed",
        "Successfully migrated database schema to v2 with FTS5 search index.",
        Some("Detailed migration logs: zero foreign key violations detected.".to_string()),
        Some("2026-10-30T12:00:00Z".to_string()),
        vec!["migration".to_string()],
    )
    .unwrap();
    storage.save_log_entry(&log).unwrap();

    // 5. Add Saved Command
    let cmd = SavedCommand::new(
        project.id,
        "Run Workspace Tests",
        "cargo test --workspace",
        Some("Runs all Rust unit and integration test suites.".to_string()),
        Some("./crates".to_string()),
        vec!["testing".to_string()],
    )
    .unwrap();
    storage.save_command(&cmd).unwrap();

    (project, storage)
}

#[test]
fn test_json_export_and_import_roundtrip() {
    let dir1 = tempdir().unwrap();
    let root1 = dir1.path();
    let (_p1, s1) = setup_sample_project(root1);

    // Export to JSON
    let export_file = root1.join("export.json");
    let options = ExportOptions {
        format: ExportFormat::Json,
        output_path: Some(export_file.clone()),
        redact_paths: false,
    };
    let (bundle, path) = ExportService::export_project(root1, &options).unwrap();
    assert_eq!(path, export_file);
    assert_eq!(bundle.records.len(), 5);
    assert_eq!(bundle.schema_version, "1.0.0");
    drop(s1);

    // Import into fresh project in dir2
    let dir2 = tempdir().unwrap();
    let root2 = dir2.path();
    let (_p2, s2) = WorkspaceManager::initialize_project(root2, Some("CleanTarget")).unwrap();

    let import_opts = ImportOptions {
        on_conflict: ConflictStrategy::Skip,
        create_backup: true,
        dry_run: false,
    };

    let result = ExportService::import_bundle(root2, &export_file, &import_opts).unwrap();
    assert_eq!(result.imported_count, 5);
    assert_eq!(result.conflicts_count, 0);
    assert_eq!(result.total_incoming, 5);
    assert!(result.backup_path.is_some());

    // Verify all records in target project
    let imported_notes = s2.list_notes(_p2.id, false).unwrap();
    assert_eq!(imported_notes.len(), 1);
    assert_eq!(imported_notes[0].record.title, "Türkçe Not Başlığı 🚀");
    assert!(imported_notes[0]
        .content_markdown
        .contains("çağlayan ve öğüt"));

    let imported_decs = s2.list_decisions(_p2.id).unwrap();
    assert_eq!(imported_decs.len(), 1);
    assert_eq!(
        imported_decs[0].record.title,
        "ADR-0013: SQLite WAL Storage"
    );

    let imported_ms = s2.list_milestones(_p2.id).unwrap();
    assert_eq!(imported_ms.len(), 1);
    assert_eq!(imported_ms[0].record.title, "Release v1.0");

    let imported_logs = s2.list_log_entries(_p2.id).unwrap();
    assert_eq!(imported_logs.len(), 1);
    assert_eq!(imported_logs[0].record.title, "Migration Completed");

    let imported_cmds = s2.list_commands(_p2.id).unwrap();
    assert_eq!(imported_cmds.len(), 1);
    assert_eq!(imported_cmds[0].command_text, "cargo test --workspace");

    drop(s2);
}

#[test]
fn test_zip_archive_export_and_import_roundtrip() {
    let dir1 = tempdir().unwrap();
    let root1 = dir1.path();
    let (_p1, s1) = setup_sample_project(root1);

    // Export to ZIP archive
    let zip_file = root1.join("bundle.zip");
    let options = ExportOptions {
        format: ExportFormat::Zip,
        output_path: Some(zip_file.clone()),
        redact_paths: true,
    };
    let (bundle, path) = ExportService::export_project(root1, &options).unwrap();
    assert_eq!(path, zip_file);
    assert_eq!(bundle.records.len(), 5);
    assert!(zip_file.exists());
    drop(s1);

    // Inspect the zip bundle
    let inspect_report = ExportService::inspect_source(root1, Some(&zip_file)).unwrap();
    assert_eq!(inspect_report.source_type, "zip_archive");
    assert_eq!(inspect_report.total_records, 5);
    assert_eq!(inspect_report.notes_count, 1);
    assert_eq!(inspect_report.decisions_count, 1);

    // Import ZIP into clean project in dir2
    let dir2 = tempdir().unwrap();
    let root2 = dir2.path();
    let (_p2, s2) = WorkspaceManager::initialize_project(root2, Some("ZipImportProject")).unwrap();

    let import_opts = ImportOptions {
        on_conflict: ConflictStrategy::Skip,
        create_backup: true,
        dry_run: false,
    };
    let result = ExportService::import_bundle(root2, &zip_file, &import_opts).unwrap();
    assert_eq!(result.imported_count, 5);
    assert_eq!(result.conflicts_count, 0);

    let notes = s2.list_notes(_p2.id, false).unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].record.title, "Türkçe Not Başlığı 🚀");
    drop(s2);
}

#[test]
fn test_markdown_export_structure() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let (_p, s) = setup_sample_project(root);

    let md_dir = root.join("markdown_export");
    let options = ExportOptions {
        format: ExportFormat::Markdown,
        output_path: Some(md_dir.clone()),
        redact_paths: false,
    };
    ExportService::export_project(root, &options).unwrap();

    assert!(md_dir.join("README.md").is_file());
    assert!(md_dir.join("notes").is_dir());
    assert!(md_dir.join("decisions").is_dir());
    assert!(md_dir.join("milestones").is_dir());
    assert!(md_dir.join("logs").is_dir());
    assert!(md_dir.join("commands").is_dir());

    let readme = fs::read_to_string(md_dir.join("README.md")).unwrap();
    assert!(readme.contains("DevContext Export: SampleProject"));
    assert!(readme.contains("ADR-0013: SQLite WAL Storage"));

    drop(s);
}

#[test]
fn test_zip_path_traversal_rejection() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create a malicious zip file containing an entry attempting directory traversal
    let evil_zip = root.join("evil.zip");
    let file = File::create(&evil_zip).unwrap();
    let mut zip = ZipWriter::new(file);

    let options = SimpleFileOptions::default();
    zip.start_file("../evil.txt", options).unwrap();
    zip.write_all(b"pwned").unwrap();
    zip.finish().unwrap();

    // Attempting extraction must be rejected with validation error
    let extract_dir = root.join("extracted");
    let result = extract_zip_safely(&evil_zip, &extract_dir);
    assert!(result.is_err());
    let err_str = result.err().unwrap().to_string();
    assert!(err_str.contains("Path traversal security violation"));
    assert!(!root.join("evil.txt").exists());
}

#[test]
fn test_unsupported_schema_version_rejection() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let (_p, storage) = WorkspaceManager::initialize_project(root, Some("SchemaCheck")).unwrap();

    // Create a JSON file with unsupported schema version 2.0.0
    let invalid_json = root.join("future-v2.json");
    let content = r#"{
        "schema_version": "2.0.0",
        "exported_at": "2026-10-30T12:00:00Z",
        "generator": { "name": "devcontext-core", "version": "2.0.0" },
        "project": { "id": "00000000-0000-0000-0000-000000000001", "name": "FutureApp", "created_at": "2026-10-30T12:00:00Z" },
        "records": []
    }"#;
    fs::write(&invalid_json, content).unwrap();

    let options = ImportOptions::default();
    let result = ExportService::import_bundle(root, &invalid_json, &options);
    assert!(result.is_err());
    let err_str = result.err().unwrap().to_string();
    assert!(err_str.contains("Unsupported export schema version '2.0.0'"));

    drop(storage);
}

#[test]
fn test_conflict_strategies_skip_overwrite_generate_id() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let (project, storage) =
        WorkspaceManager::initialize_project(root, Some("ConflictTest")).unwrap();

    // Original note in project
    let note = Note::new(
        project.id,
        "Original Note",
        "Original content in database.",
        vec!["original".to_string()],
        None,
    )
    .unwrap();
    storage.save_note(&note).unwrap();
    let original_id = note.record.id;

    // Export this single note
    let export_file = root.join("export.json");
    let export_opts = ExportOptions {
        format: ExportFormat::Json,
        output_path: Some(export_file.clone()),
        redact_paths: false,
    };
    ExportService::export_project(root, &export_opts).unwrap();

    // Modify export file so incoming record has modified content but same UUID
    let mut bundle = serde_json::from_str::<devcontext_core::export::ExportBundle>(
        &fs::read_to_string(&export_file).unwrap(),
    )
    .unwrap();
    bundle.records[0].title = "Incoming Modified Title".to_string();
    bundle.records[0].payload = serde_json::json!({
        "content_markdown": "Incoming updated content from bundle.",
        "category": null,
        "is_pinned": true,
        "is_archived": false
    });
    fs::write(&export_file, serde_json::to_string_pretty(&bundle).unwrap()).unwrap();

    // 1. ConflictStrategy::Skip
    let skip_opts = ImportOptions {
        on_conflict: ConflictStrategy::Skip,
        create_backup: false,
        dry_run: false,
    };
    let skip_res = ExportService::import_bundle(root, &export_file, &skip_opts).unwrap();
    assert_eq!(skip_res.skipped_count, 1);
    assert_eq!(skip_res.overwritten_count, 0);
    assert_eq!(skip_res.imported_count, 0);
    let note_after_skip = storage.get_note(original_id).unwrap().unwrap();
    assert_eq!(note_after_skip.record.title, "Original Note");

    // 2. ConflictStrategy::Overwrite
    let overwrite_opts = ImportOptions {
        on_conflict: ConflictStrategy::Overwrite,
        create_backup: false,
        dry_run: false,
    };
    let overwrite_res = ExportService::import_bundle(root, &export_file, &overwrite_opts).unwrap();
    assert_eq!(overwrite_res.overwritten_count, 1);
    assert_eq!(overwrite_res.skipped_count, 0);
    let note_after_overwrite = storage.get_note(original_id).unwrap().unwrap();
    assert_eq!(note_after_overwrite.record.title, "Incoming Modified Title");
    assert!(note_after_overwrite.is_pinned);

    // 3. ConflictStrategy::GenerateNewId
    let new_id_opts = ImportOptions {
        on_conflict: ConflictStrategy::GenerateNewId,
        create_backup: false,
        dry_run: false,
    };
    let new_id_res = ExportService::import_bundle(root, &export_file, &new_id_opts).unwrap();
    assert_eq!(new_id_res.imported_count, 1);
    let notes_total = storage.list_notes(project.id, false).unwrap();
    assert_eq!(notes_total.len(), 2);

    drop(storage);
}

#[test]
fn test_project_data_deletion_with_backup() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let (project, storage) = setup_sample_project(root);

    // Verify 5 active records
    assert_eq!(storage.list_notes(project.id, false).unwrap().len(), 1);
    assert_eq!(storage.list_decisions(project.id).unwrap().len(), 1);

    // Deletion without force must be rejected
    let no_force = ExportService::delete_project_data(root, false, true);
    assert!(no_force.is_err());

    // Deletion with force and backup
    let delete_result = ExportService::delete_project_data(root, true, true).unwrap();
    assert_eq!(delete_result.deleted_records_count, 5);
    assert!(delete_result.backup_path.is_some());

    // Verify active database is completely empty of records
    assert_eq!(storage.list_notes(project.id, false).unwrap().len(), 0);
    assert_eq!(storage.list_decisions(project.id).unwrap().len(), 0);
    assert_eq!(storage.list_milestones(project.id).unwrap().len(), 0);
    assert_eq!(storage.list_log_entries(project.id).unwrap().len(), 0);
    assert_eq!(storage.list_commands(project.id).unwrap().len(), 0);

    // Verify pre-deletion backup file exists and has valid integrity
    let backup_file = delete_result.backup_path.unwrap();
    assert!(std::path::Path::new(&backup_file).exists());
    let restored_storage =
        BackupService::restore_from_file(backup_file, root.join("restored.db")).unwrap();
    let integrity = BackupService::verify_integrity(&restored_storage).unwrap();
    assert_eq!(integrity, vec!["ok".to_string()]);
    assert_eq!(
        restored_storage
            .list_notes(project.id, false)
            .unwrap()
            .len(),
        1
    );

    drop(storage);
}
