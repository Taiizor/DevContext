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

#[test]
fn test_schema_upgrade_0_9_0_to_1_0_0() {
    let raw_0_9 = serde_json::json!({
        "schema_version": "0.9.0",
        "exported_at": "2026-10-30T10:00:00Z",
        "project": {
            "id": "11111111-1111-1111-1111-111111111111",
            "name": "LegacyProject",
            "created_at": "2026-10-30T09:00:00Z"
        },
        "records": [
            {
                "id": "22222222-2222-2222-2222-222222222222",
                "record_type": "note",
                "title": "Migrated Note",
                "created_at": "2026-10-30T09:30:00Z",
                "updated_at": "2026-10-30T09:30:00Z",
                "tags": ["legacy", "v0.9"],
                "references": [],
                "payload": {
                    "content_markdown": "Preserved content from 0.9.0 schema"
                }
            }
        ]
    });

    let upgraded = ExportService::upgrade_bundle(raw_0_9).unwrap();
    assert_eq!(upgraded.schema_version, "1.0.0");
    assert_eq!(upgraded.project.name, "LegacyProject");
    assert_eq!(upgraded.records.len(), 1);
    assert_eq!(upgraded.records[0].title, "Migrated Note");
    assert_eq!(upgraded.records[0].tags, vec!["legacy", "v0.9"]);
}

#[test]
fn test_unsupported_future_schema_version_rejection() {
    let future_schema = serde_json::json!({
        "schema_version": "2.0.0",
        "exported_at": "2027-01-01T00:00:00Z",
        "generator": { "name": "devcontext-future", "version": "2.0.0" },
        "project": {
            "id": "11111111-1111-1111-1111-111111111111",
            "name": "FutureProject",
            "created_at": "2027-01-01T00:00:00Z"
        },
        "records": []
    });

    let result = ExportService::upgrade_bundle(future_schema);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Unsupported export schema version '2.0.0'"));
    assert!(err_msg.contains("Please upgrade DevContext"));
}

#[test]
fn test_conflict_strategy_merge() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let (project, storage) = setup_sample_project(root);

    let existing_notes = storage.list_notes(project.id, false).unwrap();
    let existing_note = &existing_notes[0];
    let note_id = existing_note.record.id;

    // Create an incoming bundle with the same note ID, newer timestamp, and new tags/references
    let incoming_bundle = devcontext_core::export::ExportBundle {
        schema_version: "1.0.0".to_string(),
        exported_at: "2026-10-30T15:00:00Z".to_string(),
        generator: devcontext_core::export::GeneratorInfo::default(),
        project: devcontext_core::export::ExportProject {
            id: project.id,
            name: project.name.clone(),
            created_at: project.created_at.clone(),
            vcs: None,
        },
        records: vec![devcontext_core::export::ExportRecord {
            id: note_id,
            record_type: "note".to_string(),
            title: "Merged Newer Title 🚀".to_string(),
            created_at: existing_note.record.created_at.clone(),
            updated_at: "2029-10-30T15:00:00Z".to_string(),
            tags: vec!["incoming-tag".to_string(), "türkçe".to_string()],
            references: vec![devcontext_core::export::ExportReference {
                id: uuid::Uuid::new_v4(),
                ref_type: "url".to_string(),
                target: "https://devcontext.dev/docs".to_string(),
                label: Some("Documentation".to_string()),
                metadata: None,
            }],
            payload: serde_json::to_value(devcontext_core::export::NotePayload {
                content_markdown: "Updated newer content markdown".to_string(),
                category: None,
                is_pinned: false,
                is_archived: false,
            })
            .unwrap(),
        }],
    };

    let bundle_file = root.join("merge-bundle.json");
    fs::write(
        &bundle_file,
        serde_json::to_string(&incoming_bundle).unwrap(),
    )
    .unwrap();

    let merge_opts = ImportOptions {
        on_conflict: ConflictStrategy::Merge,
        create_backup: false,
        dry_run: false,
    };

    let res = ExportService::import_bundle(root, &bundle_file, &merge_opts).unwrap();
    assert_eq!(res.merged_count, 1);
    assert_eq!(res.conflicts_count, 1);

    // Verify merged state in storage
    let updated_note = storage.get_note(note_id).unwrap().unwrap();
    assert_eq!(updated_note.record.title, "Merged Newer Title 🚀");
    assert_eq!(
        updated_note.content_markdown,
        "Updated newer content markdown"
    );
    // Verify tags union: "türkçe", "unicode", "incoming-tag"
    assert!(updated_note.record.tags.contains(&"türkçe".to_string()));
    assert!(updated_note.record.tags.contains(&"unicode".to_string()));
    assert!(updated_note
        .record
        .tags
        .contains(&"incoming-tag".to_string()));

    // Verify reference union
    let refs = storage.list_references_for_record(note_id).unwrap();
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].target, "https://devcontext.dev/docs");

    drop(storage);
}

#[test]
fn test_conflict_strategy_replace_cleans_references() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let (project, storage) = setup_sample_project(root);

    let existing_notes = storage.list_notes(project.id, false).unwrap();
    let note_id = existing_notes[0].record.id;

    // Add initial reference
    let initial_ref = devcontext_core::domain::Reference::new(
        note_id,
        devcontext_core::domain::ReferenceType::Url,
        "https://old-link.com",
        Some("Old Link".to_string()),
        None,
    )
    .unwrap();
    storage.save_reference(&initial_ref).unwrap();
    assert_eq!(
        storage.list_references_for_record(note_id).unwrap().len(),
        1
    );

    // Incoming bundle with replacement reference
    let incoming_bundle = devcontext_core::export::ExportBundle {
        schema_version: "1.0.0".to_string(),
        exported_at: "2026-10-30T16:00:00Z".to_string(),
        generator: devcontext_core::export::GeneratorInfo::default(),
        project: devcontext_core::export::ExportProject {
            id: project.id,
            name: project.name.clone(),
            created_at: project.created_at.clone(),
            vcs: None,
        },
        records: vec![devcontext_core::export::ExportRecord {
            id: note_id,
            record_type: "note".to_string(),
            title: "Replaced Note Title".to_string(),
            created_at: existing_notes[0].record.created_at.clone(),
            updated_at: "2026-10-30T16:00:00Z".to_string(),
            tags: vec!["replaced".to_string()],
            references: vec![devcontext_core::export::ExportReference {
                id: uuid::Uuid::new_v4(),
                ref_type: "url".to_string(),
                target: "https://new-replacement-link.com".to_string(),
                label: Some("New Link".to_string()),
                metadata: None,
            }],
            payload: serde_json::to_value(devcontext_core::export::NotePayload {
                content_markdown: "Replaced content".to_string(),
                category: None,
                is_pinned: false,
                is_archived: false,
            })
            .unwrap(),
        }],
    };

    let bundle_file = root.join("replace-bundle.json");
    fs::write(
        &bundle_file,
        serde_json::to_string(&incoming_bundle).unwrap(),
    )
    .unwrap();

    let overwrite_opts = ImportOptions {
        on_conflict: ConflictStrategy::Overwrite,
        create_backup: false,
        dry_run: false,
    };

    let res = ExportService::import_bundle(root, &bundle_file, &overwrite_opts).unwrap();
    assert_eq!(res.overwritten_count, 1);

    // Old reference should be replaced, NOT duplicated
    let refs = storage.list_references_for_record(note_id).unwrap();
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].target, "https://new-replacement-link.com");

    drop(storage);
}

#[test]
fn test_windows_ads_colon_and_null_byte_rejection() {
    use devcontext_core::export::is_safe_archive_path;
    use std::path::Path;

    // Windows Alternate Data Streams (ADS)
    assert!(!is_safe_archive_path(Path::new("normal.txt:evil.exe")));
    assert!(!is_safe_archive_path(Path::new(
        "subfolder/data.txt::$DATA"
    )));

    // Null bytes
    assert!(!is_safe_archive_path(Path::new("malicious\0file.txt")));

    // UNC Network paths
    assert!(!is_safe_archive_path(Path::new(
        "//remote-server/share/file.txt"
    )));

    // Standard safe path
    assert!(is_safe_archive_path(Path::new("notes/valid-note.md")));
}

#[test]
fn test_corrupted_truncated_archive_rejection() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let corrupt_zip = root.join("corrupted.zip");

    // Write incomplete header bytes
    fs::write(&corrupt_zip, b"PK\x03\x04incomplete_header_bytes").unwrap();

    let extract_dir = root.join("extract_sandbox");
    let res = extract_zip_safely(&corrupt_zip, &extract_dir);
    assert!(res.is_err());
}

#[test]
fn test_import_atomicity_and_rollback_on_failure() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let (project, storage) = setup_sample_project(root);

    // Active project starts with 1 note
    assert_eq!(storage.list_notes(project.id, false).unwrap().len(), 1);

    // Construct a bundle where record 1 is valid, but record 2 has an invalid payload
    let broken_bundle = serde_json::json!({
        "schema_version": "1.0.0",
        "exported_at": "2026-10-30T17:00:00Z",
        "generator": { "name": "devcontext-test", "version": "0.1.0" },
        "project": {
            "id": project.id.to_string(),
            "name": project.name,
            "created_at": project.created_at
        },
        "records": [
            {
                "id": uuid::Uuid::new_v4().to_string(),
                "record_type": "note",
                "title": "Valid Record 1",
                "created_at": "2026-10-30T17:00:00Z",
                "updated_at": "2026-10-30T17:00:00Z",
                "tags": [],
                "references": [],
                "payload": {
                    "content_markdown": "Valid markdown note"
                }
            },
            {
                "id": uuid::Uuid::new_v4().to_string(),
                "record_type": "note",
                "title": "Broken Record 2",
                "created_at": "2026-10-30T17:01:00Z",
                "updated_at": "2026-10-30T17:01:00Z",
                "tags": [],
                "references": [],
                "payload": "BROKEN_STRING_INSTEAD_OF_OBJECT"
            }
        ]
    });

    let broken_file = root.join("broken.json");
    fs::write(&broken_file, serde_json::to_string(&broken_bundle).unwrap()).unwrap();

    let opts = ImportOptions {
        on_conflict: ConflictStrategy::Skip,
        create_backup: true,
        dry_run: false,
    };

    let import_res = ExportService::import_bundle(root, &broken_file, &opts);
    assert!(import_res.is_err());

    // Verify atomicity: Record 1 was rolled back; active project still has only the 1 original note!
    assert_eq!(storage.list_notes(project.id, false).unwrap().len(), 1);

    drop(storage);
}

#[test]
fn test_saved_command_working_directory_sanitization() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let (project, storage) =
        WorkspaceManager::initialize_project(root, Some("PrivacyProject")).unwrap();

    // Add command with sensitive user path in working directory
    let cmd = SavedCommand::new(
        project.id,
        "Sensitive Build",
        "cargo build",
        None,
        Some("C:/Users/alice/projects/secret_repo".to_string()),
        vec![],
    )
    .unwrap();
    storage.save_command(&cmd).unwrap();

    let export_opts = ExportOptions {
        format: ExportFormat::Json,
        output_path: None,
        redact_paths: false, // Default privacy mode
    };

    let (bundle, _) = ExportService::export_project(root, &export_opts).unwrap();
    let exported_cmd = &bundle.records[0];
    let payload: devcontext_core::export::SavedCommandPayload =
        serde_json::from_value(exported_cmd.payload.clone()).unwrap();

    // Verify username "alice" is redacted
    let wd = payload.working_directory.unwrap();
    assert!(!wd.contains("alice"));
    assert!(wd.contains("<redacted>"));

    drop(storage);
}

#[test]
fn test_multiple_backups_preservation_and_uniqueness() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let (_project, storage) = setup_sample_project(root);

    let export_file = dir.path().join("export.json");
    let export_opts = ExportOptions {
        format: ExportFormat::Json,
        output_path: Some(export_file.clone()),
        redact_paths: false,
    };
    let (_bundle, _path) = ExportService::export_project(root, &export_opts).unwrap();
    drop(storage);

    let import_opts = ImportOptions {
        on_conflict: ConflictStrategy::Skip,
        create_backup: true,
        dry_run: false,
    };

    // Operation 1: Import with backup
    let res1 = ExportService::import_bundle(root, &export_file, &import_opts).unwrap();
    assert!(res1.backup_path.is_some());

    // Small delay to ensure timestamp variance if needed, but UUID already guarantees uniqueness
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Operation 2: Delete project data with backup
    let res2 = ExportService::delete_project_data(root, true, true).unwrap();
    assert!(res2.backup_path.is_some());

    std::thread::sleep(std::time::Duration::from_millis(50));

    // Operation 3: Import again with backup
    let res3 = ExportService::import_bundle(root, &export_file, &import_opts).unwrap();
    assert!(res3.backup_path.is_some());

    let backup_dir = root.join(".devcontext").join("backups");
    let mut backup_entries: Vec<std::path::PathBuf> = fs::read_dir(&backup_dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|ent| ent.path()))
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("db"))
        .collect();
    backup_entries.sort();

    // Verify exactly 3 distinct, preserved backups
    assert_eq!(
        backup_entries.len(),
        3,
        "All previous backups must be preserved"
    );

    // Verify all filenames are unique
    let filenames: std::collections::HashSet<String> = backup_entries
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    assert_eq!(filenames.len(), 3, "Each backup must have a unique name");

    // Verify each backup file passes strict SQLite integrity verification
    for bp in &backup_entries {
        assert!(
            BackupService::verify_file_integrity(bp).is_ok(),
            "Backup {} must pass integrity check",
            bp.display()
        );
    }
}

#[test]
fn test_backup_destination_already_exists_rejection() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let (_project, storage) = setup_sample_project(root);

    let backup_path = dir.path().join("existing_backup.db");
    fs::write(&backup_path, b"some existing file").unwrap();

    let res = BackupService::backup_to_file(&storage, &backup_path);
    assert!(res.is_err(), "Must reject overwriting existing backup file");
    let err_str = res.unwrap_err().to_string();
    assert!(err_str.contains("already exists"));
}

#[test]
fn test_backup_creation_failure_aborts_destructive_operation() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let (project, storage) = setup_sample_project(root);
    assert_eq!(storage.list_notes(project.id, false).unwrap().len(), 1);
    drop(storage);

    // Block backup directory by creating a regular file at .devcontext/backups
    let backup_dir_blocker = root.join(".devcontext").join("backups");
    fs::write(&backup_dir_blocker, b"blocking directory creation").unwrap();

    // Attempt delete_project_data with backup requested
    let res = ExportService::delete_project_data(root, true, true);
    assert!(
        res.is_err(),
        "Destructive deletion must abort if backup creation fails"
    );

    // Verify project data was NOT deleted
    let (_project2, storage2) = WorkspaceManager::open_project(root).unwrap();
    assert_eq!(
        storage2.list_notes(project.id, false).unwrap().len(),
        1,
        "Records must remain intact when backup fails"
    );
}

#[test]
fn test_backup_filename_collision_handling_and_preservation() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let (_project, storage) = setup_sample_project(root);

    let backup_dir = root.join(".devcontext").join("backups");
    fs::create_dir_all(&backup_dir).unwrap();

    // Create a pre-existing file in the backup directory
    let existing_file = backup_dir.join("pre-existing-backup.db");
    fs::write(&existing_file, b"PRESERVED_ORIGINAL_BACKUP_CONTENT").unwrap();

    // Call atomic backup creation
    let new_backup =
        BackupService::create_atomic_backup_file(&storage, &backup_dir, "pre-import").unwrap();

    // Verify the newly created backup exists and passes integrity check
    assert!(new_backup.exists());
    assert_ne!(new_backup, existing_file);
    assert!(BackupService::verify_file_integrity(&new_backup).is_ok());

    // Verify the original pre-existing file was NEVER overwritten or deleted
    let original_bytes = fs::read(&existing_file).unwrap();
    assert_eq!(original_bytes, b"PRESERVED_ORIGINAL_BACKUP_CONTENT");
}

#[test]
fn test_sqlite_restore_failure_preserves_pre_restore_database() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let (project, storage) = setup_sample_project(root);

    // Initial note exists
    let initial_notes = storage.list_notes(project.id, false).unwrap();
    assert_eq!(initial_notes.len(), 1);
    let note1_id = initial_notes[0].record.id;

    // Add reference to initial note
    let ref1 = devcontext_core::domain::Reference::new(
        note1_id,
        devcontext_core::domain::ReferenceType::Url,
        "https://example.com/initial-ref",
        Some("Initial Ref".to_string()),
        None,
    )
    .unwrap();
    storage.save_reference(&ref1).unwrap();

    // Create a valid backup of this initial state
    let backup_dir = root.join(".devcontext").join("backups");
    let valid_backup =
        BackupService::create_atomic_backup_file(&storage, &backup_dir, "snapshot").unwrap();

    // Now mutate database: add a second note and reference after the backup
    let note2 = Note::new(
        project.id,
        "Pre-Restore Note Added",
        "Content that must remain preserved if restore fails",
        vec!["pre-restore".to_string()],
        None,
    )
    .unwrap();
    storage.save_note(&note2).unwrap();
    let ref2 = devcontext_core::domain::Reference::new(
        note2.record.id,
        devcontext_core::domain::ReferenceType::GitCommit,
        "a1b2c3d4e5f6",
        Some("Commit Ref".to_string()),
        None,
    )
    .unwrap();
    storage.save_reference(&ref2).unwrap();

    assert_eq!(storage.list_notes(project.id, false).unwrap().len(), 2);
    assert_eq!(
        storage.list_references_for_record(note1_id).unwrap().len(),
        1
    );
    assert_eq!(
        storage
            .list_references_for_record(note2.record.id)
            .unwrap()
            .len(),
        1
    );

    // Inject failure during restore: exactly 1 page batch is transferred before failure is triggered
    let restore_res =
        BackupService::restore_to_storage_with_injected_failure(&valid_backup, &storage);
    assert!(
        restore_res.is_err(),
        "Must fail when failure is injected during restore"
    );

    let err_msg = restore_res.unwrap_err().to_string();
    assert!(
        err_msg.contains("after writing 1 page"),
        "Error message must prove write occurred before failure: {}",
        err_msg
    );
    assert!(
        err_msg.contains("remaining:"),
        "Error message must contain remaining page count: {}",
        err_msg
    );
    assert!(
        err_msg.contains("total:"),
        "Error message must contain total page count: {}",
        err_msg
    );

    // 1. Verify original database remains valid and all pre-restore records and references remain accessible
    let notes_after_failed_restore = storage.list_notes(project.id, false).unwrap();
    assert_eq!(
        notes_after_failed_restore.len(),
        2,
        "Pre-restore records must remain accessible after failed restore"
    );
    let refs1_after = storage.list_references_for_record(note1_id).unwrap();
    assert_eq!(refs1_after.len(), 1);
    assert_eq!(refs1_after[0].target, "https://example.com/initial-ref");

    let refs2_after = storage.list_references_for_record(note2.record.id).unwrap();
    assert_eq!(refs2_after.len(), 1);
    assert_eq!(refs2_after[0].target, "a1b2c3d4e5f6");

    // 2. Verify PRAGMA integrity_check succeeds on storage
    let integrity = BackupService::verify_integrity(&storage).unwrap();
    assert_eq!(integrity, vec!["ok".to_string()]);

    // 3. Verify active storage connection remains usable by performing an insert
    let note3 = Note::new(
        project.id,
        "Post-Failed-Restore Note",
        "Connection is still alive and usable",
        vec![],
        None,
    )
    .unwrap();
    storage.save_note(&note3).unwrap();
    let ref3 = devcontext_core::domain::Reference::new(
        note3.record.id,
        devcontext_core::domain::ReferenceType::File,
        "src/main.rs",
        Some("File Ref".to_string()),
        None,
    )
    .unwrap();
    storage.save_reference(&ref3).unwrap();

    assert_eq!(storage.list_notes(project.id, false).unwrap().len(), 3);
    assert_eq!(
        storage
            .list_references_for_record(note3.record.id)
            .unwrap()
            .len(),
        1
    );

    // 4. Verify no unintended temporary files remain in .devcontext
    let devcontext_dir = root.join(".devcontext");
    for entry in fs::read_dir(&devcontext_dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy();
        assert!(
            !name.starts_with(".restore-staging-"),
            "No temporary staging files should remain: {}",
            name
        );
        assert!(
            !name.starts_with(".restore-orig-"),
            "No temporary orig swap files should remain: {}",
            name
        );
    }
}

#[test]
fn test_sqlite_rollback_failure_preserves_emergency_snapshot_and_dual_error() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let (project, storage) = setup_sample_project(root);

    let initial_notes = storage.list_notes(project.id, false).unwrap();
    assert_eq!(initial_notes.len(), 1);

    // Create a valid backup
    let backup_dir = root.join(".devcontext").join("backups");
    let valid_backup =
        BackupService::create_atomic_backup_file(&storage, &backup_dir, "snapshot").unwrap();

    // Call restore with injected failure during BOTH restore and rollback
    let res =
        BackupService::restore_to_storage_with_injected_rollback_failure(&valid_backup, &storage);

    assert!(res.is_err(), "Must return Err when rollback also fails");
    let err_msg = res.unwrap_err().to_string();

    // Verify dual-error message and manual recovery instructions
    assert!(
        err_msg.contains("CRITICAL: Restore failed"),
        "Error must report CRITICAL restore failure: {}",
        err_msg
    );
    assert!(
        err_msg.contains("Injected restore failure after writing 1 page"),
        "Original restore error must be preserved: {}",
        err_msg
    );
    assert!(
        err_msg.contains("AND automatic rollback also failed"),
        "Rollback failure must be stated: {}",
        err_msg
    );
    assert!(
        err_msg.contains("Injected rollback stepping failure"),
        "Rollback root error must be preserved: {}",
        err_msg
    );
    assert!(
        err_msg.contains("Database is in an unverified state and requires manual recovery"),
        "Must inform user that manual recovery is required: {}",
        err_msg
    );
    assert!(
        err_msg.contains("Emergency safety snapshot preserved at:"),
        "Must specify location of emergency snapshot: {}",
        err_msg
    );

    // Extract the emergency snapshot path from the error message
    let marker = "Emergency safety snapshot preserved at: ";
    let idx = err_msg.find(marker).expect("marker must be found");
    let preserved_path_str = &err_msg[idx + marker.len()..];
    let preserved_path = std::path::Path::new(preserved_path_str.trim());

    // Verify the emergency file physically exists on disk and is non-empty
    assert!(
        preserved_path.exists(),
        "Emergency snapshot file must exist on disk at {}",
        preserved_path_str
    );
    let metadata = fs::metadata(preserved_path).unwrap();
    assert!(
        metadata.len() > 0,
        "Emergency snapshot must be a valid non-empty file"
    );

    // Verify file integrity of the emergency snapshot
    assert!(
        BackupService::verify_file_integrity(preserved_path).is_ok(),
        "Emergency snapshot must pass integrity check"
    );

    // Verify that data can be read from the emergency snapshot file
    let conn = rusqlite::Connection::open(preserved_path).unwrap();
    let note_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
        .unwrap();
    assert_eq!(
        note_count, 1,
        "Emergency snapshot must preserve the original database content"
    );
}

#[test]
fn test_backup_creation_failure_aborts_import_operation() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let (project, storage) = setup_sample_project(root);

    // Initial state: 1 note
    assert_eq!(storage.list_notes(project.id, false).unwrap().len(), 1);

    // Export current state to file
    let export_file = dir.path().join("export_for_import_test.json");
    let export_opts = ExportOptions {
        format: ExportFormat::Json,
        output_path: Some(export_file.clone()),
        redact_paths: false,
    };
    let (_bundle, _) = ExportService::export_project(root, &export_opts).unwrap();
    drop(storage);

    // Force backup creation to fail by placing a regular file at .devcontext/backups
    let backup_dir_blocker = root.join(".devcontext").join("backups");
    fs::write(&backup_dir_blocker, b"blocker").unwrap();

    let import_opts = ImportOptions {
        on_conflict: ConflictStrategy::Skip,
        create_backup: true,
        dry_run: false,
    };

    // Attempt import with backup requested
    let res = ExportService::import_bundle(root, &export_file, &import_opts);
    assert!(
        res.is_err(),
        "Import must abort before mutation if backup creation fails"
    );

    // Verify existing records remain completely unchanged
    let (_p2, storage2) = WorkspaceManager::open_project(root).unwrap();
    let notes = storage2.list_notes(project.id, false).unwrap();
    assert_eq!(
        notes.len(),
        1,
        "Record count must remain unchanged when import aborts due to backup failure"
    );
    assert_eq!(notes[0].record.title, "Türkçe Not Başlığı 🚀");
}
