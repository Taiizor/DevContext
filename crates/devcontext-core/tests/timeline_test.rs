use devcontext_core::domain::{
    Decision, DecisionStatus, Milestone, MilestoneStatus, Note, Project, Reference, ReferenceType,
};
use devcontext_core::git::{GitAdapter, TimelineEventType, TimelineFilter, TimelineService};
use devcontext_core::storage::SqliteStorage;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

fn init_git_repo(path: &Path) {
    Command::new("git")
        .arg("init")
        .current_dir(path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["config", "user.name", "Timeline Tester"])
        .current_dir(path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["config", "user.email", "timeline@devcontext.local"])
        .current_dir(path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["config", "commit.gpgsign", "false"])
        .current_dir(path)
        .output()
        .unwrap();
}

fn create_commit(path: &Path, file_name: &str, content: &str, message: &str) -> String {
    let file_path = path.join(file_name);
    let mut file = File::create(&file_path).unwrap();
    file.write_all(content.as_bytes()).unwrap();

    Command::new("git")
        .args(["add", file_name])
        .current_dir(path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(path)
        .output()
        .unwrap();

    let out = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(path)
        .output()
        .unwrap();

    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn test_timeline_in_non_git_project() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let storage = SqliteStorage::open_in_memory().unwrap();
    let project = Project::new("Non-Git App", None).unwrap();
    storage.save_project(&project).unwrap();

    let note = Note::new(project.id, "Note 1", "Body 1", vec![], None).unwrap();
    storage.save_note(&note).unwrap();

    let decision = Decision::new(
        project.id,
        "ADR 0001",
        DecisionStatus::Accepted,
        "Ctx",
        "Dec",
        "Cons",
        None,
        vec![],
    )
    .unwrap();
    storage.save_decision(&decision).unwrap();

    let filter = TimelineFilter::default();
    let events = TimelineService::assemble_timeline(&storage, project.id, root, &filter).unwrap();

    // Must assemble DevContext records cleanly even though it's not a git repository
    assert_eq!(events.len(), 2);
    assert!(events
        .iter()
        .any(|e| e.event_type == TimelineEventType::Note));
    assert!(events
        .iter()
        .any(|e| e.event_type == TimelineEventType::Decision));
}

#[test]
fn test_combined_timeline_with_git_commits_and_filters() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    init_git_repo(root);

    let storage = SqliteStorage::open_in_memory().unwrap();
    let project = Project::new("Combined App", None).unwrap();
    storage.save_project(&project).unwrap();

    // 1. Git Commit 1
    let c1 = create_commit(root, "c1.txt", "1", "feat: initial commit");

    // 2. DevContext Note
    let mut note = Note::new(
        project.id,
        "Context Note",
        "Explaining c1",
        vec!["docs".into()],
        None,
    )
    .unwrap();
    // Anchor timestamp after c1
    note.record.created_at = "2026-10-30T10:00:00Z".to_string();
    storage.save_note(&note).unwrap();

    // 3. Git Commit 2
    let c2 = create_commit(root, "c2.txt", "2", "fix: database index");

    // 4. Milestone
    let mut milestone = Milestone::new(
        project.id,
        "Milestone Alpha",
        MilestoneStatus::Completed,
        None,
        None,
        vec![],
    )
    .unwrap();
    milestone.record.created_at = "2026-10-30T12:00:00Z".to_string();
    storage.save_milestone(&milestone).unwrap();

    // Full Timeline
    let filter = TimelineFilter::default();
    let events = TimelineService::assemble_timeline(&storage, project.id, root, &filter).unwrap();
    assert_eq!(events.len(), 4);

    // Verify chronological ordering: newest event at index 0
    for i in 0..events.len() - 1 {
        assert!(events[i].timestamp >= events[i + 1].timestamp);
    }

    // Filter by type: Commit only
    let commit_filter = TimelineFilter {
        event_type: Some(TimelineEventType::Commit),
        ..Default::default()
    };
    let commit_events =
        TimelineService::assemble_timeline(&storage, project.id, root, &commit_filter).unwrap();
    assert_eq!(commit_events.len(), 2);
    assert!(commit_events
        .iter()
        .all(|e| e.event_type == TimelineEventType::Commit));

    // Filter by limit: 1 event
    let limit_filter = TimelineFilter {
        limit: Some(1),
        ..Default::default()
    };
    let limited =
        TimelineService::assemble_timeline(&storage, project.id, root, &limit_filter).unwrap();
    assert_eq!(limited.len(), 1);

    // Check commit references
    let ref_commit = Reference::new(
        note.record.id,
        ReferenceType::GitCommit,
        c1.clone(),
        Some("Initial Commit Reference".into()),
        None,
    )
    .unwrap();
    storage.save_reference(&ref_commit).unwrap();

    assert!(GitAdapter::commit_exists(root, &c1));
    assert!(GitAdapter::commit_exists(root, &c2));
    assert!(!GitAdapter::commit_exists(
        root,
        "0000000000000000000000000000000000000000"
    ));
}

#[test]
fn test_timeline_stable_ordering_for_identical_timestamps() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let storage = SqliteStorage::open_in_memory().unwrap();
    let project = Project::new("TieBreaker App", None).unwrap();
    storage.save_project(&project).unwrap();

    let fixed_timestamp = "2026-10-30T15:00:00Z".to_string();

    let mut n1 = Note::new(project.id, "Note Tie 1", "Body", vec![], None).unwrap();
    n1.record.created_at = fixed_timestamp.clone();
    storage.save_note(&n1).unwrap();

    let mut n2 = Note::new(project.id, "Note Tie 2", "Body", vec![], None).unwrap();
    n2.record.created_at = fixed_timestamp.clone();
    storage.save_note(&n2).unwrap();

    let mut d1 = Decision::new(
        project.id,
        "ADR Tie",
        DecisionStatus::Accepted,
        "Ctx",
        "Dec",
        "Cons",
        None,
        vec![],
    )
    .unwrap();
    d1.record.created_at = fixed_timestamp.clone();
    storage.save_decision(&d1).unwrap();

    let events1 =
        TimelineService::assemble_timeline(&storage, project.id, root, &TimelineFilter::default())
            .unwrap();
    let events2 =
        TimelineService::assemble_timeline(&storage, project.id, root, &TimelineFilter::default())
            .unwrap();

    // Verify ordering is 100% deterministic between runs
    assert_eq!(events1, events2);
    // Decision has higher type priority than Note
    assert_eq!(events1[0].event_type, TimelineEventType::Decision);
}
