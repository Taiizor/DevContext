use devcontext_core::error::Result;
use devcontext_core::storage::connection::DbConnection;
use devcontext_core::storage::migrations::{Migration, MigrationRunner};
use rusqlite::Transaction;

struct FailingMigration;
impl Migration for FailingMigration {
    fn version(&self) -> i64 {
        999
    }
    fn name(&self) -> &'static str {
        "999_failing_migration"
    }
    fn up(&self, tx: &Transaction) -> Result<()> {
        tx.execute_batch("CREATE TABLE temp_table (id INTEGER);")?;
        // Intentionally invalid statement to trigger error
        tx.execute_batch("CREATE INVALID SYNTAX;")?;
        Ok(())
    }
}

#[test]
fn test_migration_lifecycle_and_idempotence() {
    let mut conn = DbConnection::open_in_memory().unwrap();

    let initial_version = MigrationRunner::current_version(&conn).unwrap();
    assert_eq!(initial_version, 0);

    let applied = MigrationRunner::run_pending(&mut conn).unwrap();
    assert_eq!(applied, 1);

    let new_version = MigrationRunner::current_version(&conn).unwrap();
    assert_eq!(new_version, 1);

    let re_applied = MigrationRunner::run_pending(&mut conn).unwrap();
    assert_eq!(re_applied, 0);

    MigrationRunner::verify_integrity(&conn).unwrap();
}

#[test]
fn test_migration_atomic_rollback_on_failure() {
    let mut conn = DbConnection::open_in_memory().unwrap();
    MigrationRunner::run_pending(&mut conn).unwrap();
    let version_before = MigrationRunner::current_version(&conn).unwrap();

    // Attempt to run the failing migration manually inside a transaction
    let tx = conn.transaction().unwrap();
    let failing = FailingMigration;
    let res = failing.up(&tx);
    assert!(res.is_err());
    drop(tx); // Rolled back

    // Version remains at 1
    let version_after = MigrationRunner::current_version(&conn).unwrap();
    assert_eq!(version_after, version_before);

    // Verify temp_table was rolled back
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='temp_table';",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map(|count| count > 0)
        .unwrap();
    assert!(!table_exists);
}
