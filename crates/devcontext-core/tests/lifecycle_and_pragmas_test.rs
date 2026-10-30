use devcontext_core::storage::{DbConnection, SqliteStorage};
use tempfile::tempdir;

#[test]
fn test_database_creation_and_reopening() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");

    // 1. Creation
    {
        let storage = SqliteStorage::open(&db_path).expect("Failed to create storage");
        drop(storage);
    }
    assert!(db_path.exists());

    // 2. Reopening existing database
    {
        let storage = SqliteStorage::open(&db_path).expect("Failed to reopen storage");
        let projects = storage.list_projects().expect("Failed to list projects");
        assert_eq!(projects.len(), 0);
    }
}

#[test]
fn test_wal_mode_and_foreign_keys_on_disk() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("wal_test.db");

    let _storage = SqliteStorage::open(&db_path).unwrap();

    let conn = DbConnection::open(&db_path).unwrap();
    let journal_mode = DbConnection::get_journal_mode(&conn).unwrap();
    assert_eq!(journal_mode, "wal");

    DbConnection::verify_foreign_keys(&conn).unwrap();
}
