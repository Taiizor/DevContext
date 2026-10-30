//! Phase 8: Quality, Workspace Edge Cases, and Database Reliability Tests.

use devcontext_core::domain::{Decision, DecisionStatus, Note};
use devcontext_core::export::{
    ConflictStrategy, ExportFormat, ExportOptions, ExportService, ImportOptions,
};
use devcontext_core::search::{SearchFilter, SearchService};
use devcontext_core::storage::backup::BackupService;
use devcontext_core::workspace::WorkspaceManager;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_workspace_with_spaces_in_path() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("path with spaces in directory name");
    fs::create_dir_all(&root).unwrap();

    let (project, storage) =
        WorkspaceManager::initialize_project(&root, Some("Spaces Project")).unwrap();
    assert_eq!(project.name, "Spaces Project");

    // Add note
    let note = Note::new(
        project.id,
        "Note in Space Path",
        "Body content for spaces path test",
        vec!["spaces".to_string()],
        None,
    )
    .unwrap();
    storage.save_note(&note).unwrap();

    // Verify retrieval
    let notes = storage.list_notes(project.id, false).unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].record.title, "Note in Space Path");

    // Verify search in workspace with spaces
    let filter = SearchFilter::default();
    let search_res =
        SearchService::search(&storage, project.id, &root, "content", &filter).unwrap();
    assert_eq!(search_res.len(), 1);

    // Verify export in workspace with spaces
    let export_file = root.join("export file with spaces.json");
    let export_opts = ExportOptions {
        format: ExportFormat::Json,
        output_path: Some(export_file.clone()),
        redact_paths: true,
    };
    let (bundle, _) = ExportService::export_project(&root, &export_opts).unwrap();
    assert_eq!(bundle.records.len(), 1);
    assert!(export_file.exists());

    // Verify import in workspace with spaces
    let import_opts = ImportOptions {
        on_conflict: ConflictStrategy::Skip,
        create_backup: true,
        dry_run: false,
    };
    let import_res = ExportService::import_bundle(&root, &export_file, &import_opts).unwrap();
    assert_eq!(import_res.skipped_count, 1);
}

#[test]
fn test_workspace_with_unicode_and_turkish_path() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("türkçe_dizin_şeker_ılık_çağlayan_öğüt_🚀");
    fs::create_dir_all(&root).unwrap();

    let (project, storage) =
        WorkspaceManager::initialize_project(&root, Some("Türkçe Proje 🚀")).unwrap();
    assert_eq!(project.name, "Türkçe Proje 🚀");

    // Add note with Unicode
    let note = Note::new(
        project.id,
        "Başlık: Türkçe Karakterler İ, Ş, Ğ, Ü, Ç, Ö 🚀",
        "İçerik: Şeker, ağaç, ılık su, çağlayan ve öğüt.",
        vec!["türkçe".to_string(), "unicode".to_string()],
        None,
    )
    .unwrap();
    storage.save_note(&note).unwrap();

    // Add decision with Unicode
    let dec = Decision::new(
        project.id,
        "ADR-0014: Yerel Depolama ve Karakter Desteği",
        DecisionStatus::Accepted,
        "Gereksinim: Türkçe ve Unicode karakterlerin tam desteklenmesi.",
        "Karar: SQLite unicode61 ve UTF-8 FTS5 normalizasyonu kullanılacaktır.",
        "Sonuçlar: Türkçe aramalar büyük/küçük harf duyarsız çalışacaktır.",
        None,
        vec!["karar".to_string()],
    )
    .unwrap();
    storage.save_decision(&dec).unwrap();

    // Verify FTS5 search handles normalized Turkish queries
    let filter = SearchFilter::default();
    let results_i = SearchService::search(&storage, project.id, &root, "seker", &filter).unwrap();
    assert_eq!(results_i.len(), 1, "Should find 'Şeker' with ASCII 'seker'");

    let results_agac = SearchService::search(&storage, project.id, &root, "ağaç", &filter).unwrap();
    assert_eq!(results_agac.len(), 1, "Should find 'ağaç'");

    // Verify integrity
    let integrity = BackupService::verify_integrity(&storage).unwrap();
    assert_eq!(integrity, vec!["ok".to_string()]);
}

#[test]
fn test_non_git_workspace_lifecycle() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("plain_folder_without_git");
    fs::create_dir_all(&root).unwrap();

    // Verify root is not a git repo
    assert!(!root.join(".git").exists());

    // Project initialization works without Git
    let (project, storage) =
        WorkspaceManager::initialize_project(&root, Some("Non-Git Project")).unwrap();
    assert_eq!(project.name, "Non-Git Project");

    // Doctor status check
    let (_p, storage2) = WorkspaceManager::open_project(&root).unwrap();
    let integrity = BackupService::verify_integrity(&storage2).unwrap();
    assert_eq!(integrity, vec!["ok".to_string()]);

    // Add note and verify storage works independently
    let note = Note::new(
        project.id,
        "Standalone Note",
        "Works without Git",
        vec!["standalone".to_string()],
        None,
    )
    .unwrap();
    storage.save_note(&note).unwrap();
    assert_eq!(storage.list_notes(project.id, false).unwrap().len(), 1);
}

#[test]
fn test_multi_root_workspace_strict_isolation() {
    let dir = tempdir().unwrap();
    let root_a = dir.path().join("workspace_project_alpha");
    let root_b = dir.path().join("workspace_project_beta");
    fs::create_dir_all(&root_a).unwrap();
    fs::create_dir_all(&root_b).unwrap();

    // Initialize two distinct projects
    let (project_a, storage_a) =
        WorkspaceManager::initialize_project(&root_a, Some("Project Alpha")).unwrap();
    let (project_b, storage_b) =
        WorkspaceManager::initialize_project(&root_b, Some("Project Beta")).unwrap();

    assert_ne!(project_a.id, project_b.id);

    // Save record to Project A
    let note_a = Note::new(
        project_a.id,
        "Confidential Alpha Strategy",
        "Alpha proprietary algorithms",
        vec!["alpha".to_string()],
        None,
    )
    .unwrap();
    storage_a.save_note(&note_a).unwrap();

    // Save record to Project B
    let note_b = Note::new(
        project_b.id,
        "Beta Public Roadmap",
        "Beta community plans",
        vec!["beta".to_string()],
        None,
    )
    .unwrap();
    storage_b.save_note(&note_b).unwrap();

    // Verify Project A contains only Alpha records
    let notes_a = storage_a.list_notes(project_a.id, false).unwrap();
    assert_eq!(notes_a.len(), 1);
    assert_eq!(notes_a[0].record.title, "Confidential Alpha Strategy");

    let notes_a_cross = storage_a.list_notes(project_b.id, false).unwrap();
    assert_eq!(
        notes_a_cross.len(),
        0,
        "Storage A must not contain Project B records"
    );

    // Verify Project B contains only Beta records
    let notes_b = storage_b.list_notes(project_b.id, false).unwrap();
    assert_eq!(notes_b.len(), 1);
    assert_eq!(notes_b[0].record.title, "Beta Public Roadmap");

    let notes_b_cross = storage_b.list_notes(project_a.id, false).unwrap();
    assert_eq!(
        notes_b_cross.len(),
        0,
        "Storage B must not contain Project A records"
    );

    // Verify Search isolation
    let filter = SearchFilter::default();
    let search_alpha =
        SearchService::search(&storage_a, project_a.id, &root_a, "Alpha", &filter).unwrap();
    assert_eq!(search_alpha.len(), 1);

    let search_beta_in_a =
        SearchService::search(&storage_a, project_a.id, &root_a, "Beta", &filter).unwrap();
    assert_eq!(
        search_beta_in_a.len(),
        0,
        "Search in project A must never return project B content"
    );
}

#[test]
fn test_database_corruption_detection_and_safe_reporting() {
    let dir = tempdir().unwrap();
    let corrupted_db = dir.path().join("corrupted.db");

    // Write a file that looks like a database header but contains garbage/zeros
    let mut header = [0u8; 1024];
    header[..16].copy_from_slice(b"SQLite format 3\0");
    fs::write(&corrupted_db, header).unwrap();

    // Integrity check must detect corruption or invalid pages
    let integrity_res = BackupService::verify_file_integrity(&corrupted_db);
    assert!(
        integrity_res.is_err(),
        "Corrupted database file must fail integrity check"
    );
}

#[test]
fn test_schema_migration_idempotence_and_preservation() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // First open initializes schema migrations
    let (project, storage) =
        WorkspaceManager::initialize_project(root, Some("Migration Project")).unwrap();

    let note = Note::new(
        project.id,
        "Pre-Migration Note",
        "Data that must persist across repeated migration runs",
        vec!["persist".to_string()],
        None,
    )
    .unwrap();
    storage.save_note(&note).unwrap();
    drop(storage);

    // Reopening the project runs migration lifecycle again
    let (project2, storage2) = WorkspaceManager::open_project(root).unwrap();
    assert_eq!(project2.id, project.id);

    let notes = storage2.list_notes(project2.id, false).unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].record.title, "Pre-Migration Note");

    // Verify integrity check passes
    let integrity = BackupService::verify_integrity(&storage2).unwrap();
    assert_eq!(integrity, vec!["ok".to_string()]);
}
