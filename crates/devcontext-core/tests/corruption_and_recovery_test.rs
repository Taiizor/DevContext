use devcontext_core::domain::{Note, Project};
use devcontext_core::storage::{BackupService, SqliteStorage};
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_corrupted_database_handling() {
    let dir = tempdir().unwrap();
    let corrupt_db_path = dir.path().join("corrupt.db");

    // Write random non-SQLite bytes to corrupt the header
    {
        let mut file = File::create(&corrupt_db_path).unwrap();
        file.write_all(b"NOT A VALID SQLITE3 DATABASE FILE HEADER JUNK DATA")
            .unwrap();
    }

    // Opening a corrupt file should fail gracefully with an Error, NEVER panic
    let res = SqliteStorage::open(&corrupt_db_path);
    assert!(
        res.is_err(),
        "Expected corrupt database opening to return an error"
    );
}

#[test]
fn test_online_backup_and_full_restore_cycle() {
    let dir = tempdir().unwrap();
    let original_db = dir.path().join("live.db");
    let backup_file = dir.path().join("live.bak");
    let restored_db = dir.path().join("restored.db");

    let storage = SqliteStorage::open(&original_db).unwrap();
    let project = Project::new("Recovery Project", None).unwrap();
    storage.save_project(&project).unwrap();

    let note = Note::new(
        project.id,
        "Crucial Context",
        "This note must survive recovery.",
        vec!["critical".into()],
        None,
    )
    .unwrap();
    storage.save_note(&note).unwrap();

    // Verify integrity of live db
    let integrity = BackupService::verify_integrity(&storage).unwrap();
    assert_eq!(integrity, vec!["ok".to_string()]);

    // Perform backup
    BackupService::backup_to_file(&storage, &backup_file).unwrap();
    assert!(backup_file.exists());

    // Restore to a new location
    let restored_storage = BackupService::restore_from_file(&backup_file, &restored_db).unwrap();

    // Verify restored records
    let restored_project = restored_storage.get_project(project.id).unwrap().unwrap();
    assert_eq!(restored_project.name, "Recovery Project");

    let restored_note = restored_storage.get_note(note.record.id).unwrap().unwrap();
    assert_eq!(restored_note.record.title, "Crucial Context");
    assert_eq!(
        restored_note.content_markdown,
        "This note must survive recovery."
    );

    // Verify foreign keys and integrity on restored db
    BackupService::verify_foreign_keys(&restored_storage).unwrap();
    let restored_integrity = BackupService::verify_integrity(&restored_storage).unwrap();
    assert_eq!(restored_integrity, vec!["ok".to_string()]);
}
