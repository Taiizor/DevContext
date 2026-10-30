use devcontext_core::domain::Project;
use devcontext_core::error::{CoreError, Result};
use devcontext_core::storage::SqliteStorage;

#[test]
fn test_transactional_atomic_rollback() {
    let storage = SqliteStorage::open_in_memory().unwrap();

    let project = Project::new("Tx Project", None).unwrap();
    storage.save_project(&project).unwrap();

    // Run a transactional block that writes, then returns an Err
    let outcome: Result<()> = storage.with_transaction(|tx| {
        tx.execute(
            "INSERT INTO records (id, project_id, record_type, title, created_at, updated_at, tags, is_deleted)
             VALUES ('00000000-0000-0000-0000-000000000001', ?1, 'note', 'Pending Note', '2026-10-30T00:00:00Z', '2026-10-30T00:00:00Z', '[]', 0);",
            rusqlite::params![project.id.to_string()],
        )?;

        // Force an error to abort the transaction
        Err(CoreError::Validation("Simulated failure during transaction".to_string()))
    });

    assert!(outcome.is_err());

    // Verify record was rolled back cleanly
    let note = storage.get_note(uuid::Uuid::from_u128(1)).unwrap();
    assert!(note.is_none());
}
