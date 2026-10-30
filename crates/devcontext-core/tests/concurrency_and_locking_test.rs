use devcontext_core::domain::{Note, Project};
use devcontext_core::storage::SqliteStorage;
use std::sync::Arc;
use std::thread;
use tempfile::tempdir;

#[test]
fn test_concurrent_access_and_thread_safety() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("concurrency.db");

    let storage = SqliteStorage::open(&db_path).unwrap();
    let project = Project::new("Concurrent Project", None).unwrap();
    storage.save_project(&project).unwrap();

    let storage_arc = Arc::new(storage);
    let project_id = project.id;

    let mut handles = Vec::new();

    // Spawn 5 threads writing notes concurrently
    for i in 0..5 {
        let storage_clone = storage_arc.clone();
        let handle = thread::spawn(move || {
            let note = Note::new(
                project_id,
                format!("Concurrent Note {}", i),
                format!("Body from worker thread {}", i),
                vec!["worker".into()],
                None,
            )
            .unwrap();

            storage_clone.save_note(&note).unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Verify all 5 notes were persisted cleanly
    let notes = storage_arc.list_notes(project_id, true).unwrap();
    assert_eq!(notes.len(), 5);
}
