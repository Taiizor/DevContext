use devcontext_core::domain::{Decision, DecisionStatus, Note, Project};
use devcontext_core::storage::SqliteStorage;

#[test]
fn test_strict_project_isolation() {
    let storage = SqliteStorage::open_in_memory().unwrap();

    let project_a = Project::new("Project Alpha", None).unwrap();
    let project_b = Project::new("Project Beta", None).unwrap();

    storage.save_project(&project_a).unwrap();
    storage.save_project(&project_b).unwrap();

    // Notes for Project A
    let note_a1 = Note::new(project_a.id, "Alpha Note 1", "Body A1", vec![], None).unwrap();
    let note_a2 = Note::new(project_a.id, "Alpha Note 2", "Body A2", vec![], None).unwrap();
    storage.save_note(&note_a1).unwrap();
    storage.save_note(&note_a2).unwrap();

    // Note for Project B
    let note_b1 = Note::new(project_b.id, "Beta Note 1", "Body B1", vec![], None).unwrap();
    storage.save_note(&note_b1).unwrap();

    // Decision for Project B
    let adr_b1 = Decision::new(
        project_b.id,
        "ADR Beta",
        DecisionStatus::Accepted,
        "Context B",
        "Decision B",
        "Consequences B",
        None,
        vec![],
    )
    .unwrap();
    storage.save_decision(&adr_b1).unwrap();

    // Querying Project A must return ONLY Project A's records
    let notes_a = storage.list_notes(project_a.id, true).unwrap();
    assert_eq!(notes_a.len(), 2);
    assert!(notes_a.iter().all(|n| n.record.project_id == project_a.id));

    let decisions_a = storage.list_decisions(project_a.id).unwrap();
    assert_eq!(decisions_a.len(), 0);

    // Querying Project B must return ONLY Project B's records
    let notes_b = storage.list_notes(project_b.id, true).unwrap();
    assert_eq!(notes_b.len(), 1);
    assert_eq!(notes_b[0].record.title, "Beta Note 1");

    let decisions_b = storage.list_decisions(project_b.id).unwrap();
    assert_eq!(decisions_b.len(), 1);
    assert_eq!(decisions_b[0].record.title, "ADR Beta");
}
