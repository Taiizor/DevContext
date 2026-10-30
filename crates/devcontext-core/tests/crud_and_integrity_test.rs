use devcontext_core::domain::{
    Decision, DecisionStatus, LogEntry, Milestone, MilestoneStatus, Note, Project, Reference,
    ReferenceType, SavedCommand,
};
use devcontext_core::error::CoreError;
use devcontext_core::storage::SqliteStorage;
use uuid::Uuid;

#[test]
fn test_note_crud_and_soft_delete() {
    let storage = SqliteStorage::open_in_memory().unwrap();
    let project = Project::new("Note Test", None).unwrap();
    storage.save_project(&project).unwrap();

    let mut note = Note::new(
        project.id,
        "Architecture Overview",
        "# System Architecture\nDetails here...",
        vec!["architecture".into(), "v1".into()],
        Some("docs".into()),
    )
    .unwrap();

    // 1. Create
    storage.save_note(&note).unwrap();

    // 2. Read
    let fetched = storage.get_note(note.record.id).unwrap().unwrap();
    assert_eq!(fetched.record.title, "Architecture Overview");
    assert_eq!(
        fetched.content_markdown,
        "# System Architecture\nDetails here..."
    );
    assert_eq!(fetched.category, Some("docs".into()));
    assert_eq!(fetched.record.tags, vec!["architecture", "v1"]);

    // 3. Update
    note.update_content("Updated architecture details").unwrap();
    note.set_pinned(true);
    storage.save_note(&note).unwrap();

    let updated = storage.get_note(note.record.id).unwrap().unwrap();
    assert_eq!(updated.content_markdown, "Updated architecture details");
    assert!(updated.is_pinned);

    // 4. Soft Delete
    assert!(storage.soft_delete_record(note.record.id).unwrap());
    // Soft-deleted note should not appear in get_note or list_notes
    assert!(storage.get_note(note.record.id).unwrap().is_none());
    assert_eq!(storage.list_notes(project.id, true).unwrap().len(), 0);
}

#[test]
fn test_decision_lifecycle_and_historical_preservation() {
    let storage = SqliteStorage::open_in_memory().unwrap();
    let project = Project::new("ADR Test", None).unwrap();
    storage.save_project(&project).unwrap();

    // Initial ADR
    let mut adr1 = Decision::new(
        project.id,
        "ADR 0001: Flat-file JSON",
        DecisionStatus::Accepted,
        "Need simple storage",
        "Use JSON files",
        "Simple but hard to query",
        None,
        vec!["storage".into()],
    )
    .unwrap();
    storage.save_decision(&adr1).unwrap();

    // New ADR superseding the first
    let adr2 = Decision::new(
        project.id,
        "ADR 0002: SQLite WAL",
        DecisionStatus::Accepted,
        "JSON files lacked indexing and transactions",
        "Migrate to SQLite with WAL mode",
        "Fast, ACID transactions",
        Some("PostgreSQL, Flat JSON".into()),
        vec!["storage".into(), "database".into()],
    )
    .unwrap();
    storage.save_decision(&adr2).unwrap();

    // Mark ADR 1 as superseded by ADR 2
    adr1.supersede(adr2.record.id).unwrap();
    storage.save_decision(&adr1).unwrap();

    // Verify historical ADR 1 is intact and clearly references ADR 2
    let fetched_adr1 = storage.get_decision(adr1.record.id).unwrap().unwrap();
    assert_eq!(fetched_adr1.status, DecisionStatus::Superseded);
    assert_eq!(fetched_adr1.superseded_by, Some(adr2.record.id));
    assert_eq!(fetched_adr1.context, "Need simple storage");

    // Both ADRs exist in project list
    let decisions = storage.list_decisions(project.id).unwrap();
    assert_eq!(decisions.len(), 2);
}

#[test]
fn test_milestone_and_log_entry_persistence() {
    let storage = SqliteStorage::open_in_memory().unwrap();
    let project = Project::new("Milestone Test", None).unwrap();
    storage.save_project(&project).unwrap();

    let milestone = Milestone::new(
        project.id,
        "Phase 2 Complete",
        MilestoneStatus::Completed,
        Some("2026-10-30T22:00:00Z".into()),
        Some("All persistence primitives implemented".into()),
        vec!["milestone".into()],
    )
    .unwrap();
    storage.save_milestone(&milestone).unwrap();

    let fetched_m = storage.get_milestone(milestone.record.id).unwrap().unwrap();
    assert_eq!(fetched_m.status, MilestoneStatus::Completed);
    assert!(fetched_m.completed_date.is_some());

    let log = LogEntry::new(
        project.id,
        "Schema v1 Migration",
        "Applied initial SQLite schema cleanly",
        Some("All tables created".into()),
        None,
        vec!["log".into()],
    )
    .unwrap();
    storage.save_log_entry(&log).unwrap();

    let fetched_log = storage.get_log_entry(log.record.id).unwrap().unwrap();
    assert_eq!(fetched_log.summary, "Applied initial SQLite schema cleanly");
}

#[test]
fn test_saved_command_persistence() {
    let storage = SqliteStorage::open_in_memory().unwrap();
    let project = Project::new("Cmd Test", None).unwrap();
    storage.save_project(&project).unwrap();

    let cmd = SavedCommand::new(
        project.id,
        "Run Linter",
        "cargo clippy --all-targets --all-features -- -D warnings",
        Some("Ensures zero clippy warnings".into()),
        Some("./".into()),
        vec!["quality".into()],
    )
    .unwrap();
    storage.save_command(&cmd).unwrap();

    let fetched = storage.get_command(cmd.record.id).unwrap().unwrap();
    assert_eq!(
        fetched.command_text,
        "cargo clippy --all-targets --all-features -- -D warnings"
    );
}

#[test]
fn test_reference_linking_and_cascade_delete() {
    let storage = SqliteStorage::open_in_memory().unwrap();
    let project = Project::new("Ref Test", None).unwrap();
    storage.save_project(&project).unwrap();

    let note = Note::new(
        project.id,
        "Ref Link Note",
        "Linking file and URL references",
        vec![],
        None,
    )
    .unwrap();
    storage.save_note(&note).unwrap();

    let ref1 = Reference::new(
        note.record.id,
        ReferenceType::File,
        "src/storage/mod.rs",
        Some("Storage Entry".into()),
        None,
    )
    .unwrap();

    let ref2 = Reference::new(
        note.record.id,
        ReferenceType::Url,
        "https://sqlite.org/wal.html",
        Some("WAL Documentation".into()),
        None,
    )
    .unwrap();

    storage.save_reference(&ref1).unwrap();
    storage.save_reference(&ref2).unwrap();

    let refs = storage.list_references_for_record(note.record.id).unwrap();
    assert_eq!(refs.len(), 2);

    // Hard deleting the parent record MUST cascade-delete child references
    assert!(storage.hard_delete_record(note.record.id).unwrap());
    let refs_after = storage.list_references_for_record(note.record.id).unwrap();
    assert_eq!(refs_after.len(), 0);
}

#[test]
fn test_foreign_key_constraint_violations() {
    let storage = SqliteStorage::open_in_memory().unwrap();

    // 1. Attempting to save a note pointing to a NON-EXISTENT project MUST fail with Foreign Key violation
    let fake_project_id = Uuid::new_v4();
    let note = Note::new(fake_project_id, "Orphan Note", "Orphan body", vec![], None).unwrap();
    let res = storage.save_note(&note);
    assert!(res.is_err());

    // 2. Attempting to save a reference pointing to a NON-EXISTENT record MUST fail
    let fake_record_id = Uuid::new_v4();
    let reference =
        Reference::new(fake_record_id, ReferenceType::File, "test.txt", None, None).unwrap();
    let res = storage.save_reference(&reference);
    assert!(res.is_err());
}

#[test]
fn test_domain_validation_invariants() {
    let proj_id = Uuid::new_v4();

    // Empty note content fails
    assert!(matches!(
        Note::new(proj_id, "Title", "  ", vec![], None),
        Err(CoreError::Validation(_))
    ));

    // Empty decision statements fail
    assert!(matches!(
        Decision::new(
            proj_id,
            "ADR",
            DecisionStatus::Proposed,
            "   ",
            "decision",
            "consequences",
            None,
            vec![]
        ),
        Err(CoreError::Validation(_))
    ));

    // Empty command text fails
    assert!(matches!(
        SavedCommand::new(proj_id, "Cmd", "   ", None, None, vec![]),
        Err(CoreError::Validation(_))
    ));
}
