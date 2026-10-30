use devcontext_core::domain::{Note, Project};
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
        .expect("git init failed");

    Command::new("git")
        .args(["config", "user.name", "Verification Tester"])
        .current_dir(path)
        .output()
        .expect("git config user.name failed");

    Command::new("git")
        .args(["config", "user.email", "verify@devcontext.local"])
        .current_dir(path)
        .output()
        .expect("git config user.email failed");

    Command::new("git")
        .args(["config", "commit.gpgsign", "false"])
        .current_dir(path)
        .output()
        .expect("git config gpgsign failed");
}

fn create_commit(path: &Path, file_name: &str, content: &str, message: &str) -> String {
    let file_path = path.join(file_name);
    let mut file = File::create(&file_path).unwrap();
    file.write_all(content.as_bytes()).unwrap();

    Command::new("git")
        .args(["add", file_name])
        .current_dir(path)
        .output()
        .expect("git add failed");

    Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(path)
        .output()
        .expect("git commit failed");

    let out = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(path)
        .output()
        .expect("rev-parse failed");

    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn test_missing_git_executable_graceful_degradation() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // 1. Querying commits with a non-existent binary returns an empty Vec without crashing
    let commits = GitAdapter::get_commits_with_bin(
        "non_existent_git_binary_xyz_404",
        root,
        Some(10),
        None,
        None,
    )
    .expect("Must gracefully handle missing executable");
    assert!(commits.is_empty());

    // 2. Timeline assembly continues to serve DevContext records even if Git is absent
    let storage = SqliteStorage::open_in_memory().unwrap();
    let project = Project::new("Degradation Test", None).unwrap();
    storage.save_project(&project).unwrap();

    let note = Note::new(project.id, "Authored Note", "Still visible", vec![], None).unwrap();
    storage.save_note(&note).unwrap();

    let filter = TimelineFilter::default();
    let events = TimelineService::assemble_timeline(&storage, project.id, root, &filter).unwrap();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].title, "Authored Note");
}

#[test]
fn test_commit_parsing_multiline_quotes_brackets_and_emojis() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    init_git_repo(root);

    // Commit with multiline message, quotes, commas, brackets, and emojis
    let complex_message =
        "feat(git): add [verification] pass \"strict-mode\", with emojis! 🚀✨\n\n\
        This is a multi-line commit body with 'single' and \"double\" quotes,\n\
        brackets [like this], commas, and UTF-8 characters: ⚡, 🛡️, 🌐.\n\n\
        Co-authored-by: Agent <agent@local>";

    let hash = create_commit(root, "complex.txt", "data", complex_message);
    assert!(!hash.is_empty());

    let commits = GitAdapter::get_commits(root, None, None, None).unwrap();
    assert_eq!(commits.len(), 1);

    let commit = &commits[0];
    assert_eq!(commit.hash, hash);
    assert!(commit.message.contains("feat(git): add [verification]"));
    assert!(commit.message.contains("🚀✨"));
    assert!(commit.message.contains("brackets [like this]"));

    // Verify how TimelineService decomposes title and description
    let storage = SqliteStorage::open_in_memory().unwrap();
    let project = Project::new("Commit Parse Test", None).unwrap();
    storage.save_project(&project).unwrap();

    let events =
        TimelineService::assemble_timeline(&storage, project.id, root, &TimelineFilter::default())
            .unwrap();

    assert_eq!(events.len(), 1);
    let ev = &events[0];
    assert_eq!(ev.event_type, TimelineEventType::Commit);
    assert_eq!(
        ev.title,
        "feat(git): add [verification] pass \"strict-mode\", with emojis! 🚀✨"
    );
    assert!(ev.description.is_some());
    let desc = ev.description.as_ref().unwrap();
    assert!(desc.contains("This is a multi-line commit body"));
    assert!(desc.contains("Co-authored-by: Agent <agent@local>"));
}

#[test]
fn test_empty_commit_message_and_atypical_author() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    init_git_repo(root);

    // Commit with empty message and empty author name
    let file_path = root.join("empty_msg.txt");
    File::create(&file_path)
        .unwrap()
        .write_all(b"empty")
        .unwrap();

    Command::new("git")
        .args(["add", "empty_msg.txt"])
        .current_dir(root)
        .output()
        .unwrap();

    let commit_res = Command::new("git")
        .args([
            "-c",
            "user.name=Anonymous",
            "-c",
            "user.email=",
            "commit",
            "--allow-empty-message",
            "-m",
            "",
        ])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(commit_res.status.success());

    let commits = GitAdapter::get_commits(root, None, None, None).unwrap();
    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].author_name, "Anonymous");
    assert_eq!(commits[0].author_email, "");

    let storage = SqliteStorage::open_in_memory().unwrap();
    let project = Project::new("Empty Msg Test", None).unwrap();
    storage.save_project(&project).unwrap();

    let events =
        TimelineService::assemble_timeline(&storage, project.id, root, &TimelineFilter::default())
            .unwrap();

    assert_eq!(events.len(), 1);
    let ev = &events[0];
    // Empty message falls back to short hash
    assert_eq!(ev.title, commits[0].short_hash);
    assert_eq!(ev.author, Some("Anonymous".to_string()));
}

#[test]
fn test_shallow_clone_metadata_retrieval() {
    let upstream_dir = tempdir().unwrap();
    let upstream_root = upstream_dir.path();
    init_git_repo(upstream_root);

    // Create 3 commits upstream
    create_commit(upstream_root, "f1.txt", "1", "commit 1");
    create_commit(upstream_root, "f2.txt", "2", "commit 2");
    let c3_hash = create_commit(upstream_root, "f3.txt", "3", "commit 3 - shallow tip");

    // Shallow clone with --depth 1 into a client directory
    let shallow_dir = tempdir().unwrap();
    let shallow_root = shallow_dir.path().join("shallow_repo");

    let clone_output = Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "--no-local",
            upstream_root.to_str().unwrap(),
            shallow_root.to_str().unwrap(),
        ])
        .output()
        .expect("git clone --depth 1 failed");
    assert!(clone_output.status.success());

    // Verify GitAdapter correctly operates on the shallow clone
    assert!(GitAdapter::is_git_repository(&shallow_root));
    let head = GitAdapter::get_head_commit(&shallow_root)
        .unwrap()
        .expect("Head commit must exist");
    assert_eq!(head, c3_hash);

    let commits = GitAdapter::get_commits(&shallow_root, None, None, None).unwrap();
    // In a --depth 1 shallow clone, only the tip commit is present locally
    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].hash, c3_hash);
    assert_eq!(commits[0].message, "commit 3 - shallow tip");
}

#[test]
fn test_timeline_date_filters_boundaries() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    init_git_repo(root);

    let storage = SqliteStorage::open_in_memory().unwrap();
    let project = Project::new("Date Filter Test", None).unwrap();
    storage.save_project(&project).unwrap();

    // Event 1: Oct 28
    let mut n1 = Note::new(project.id, "Day 1 Note", "Body", vec![], None).unwrap();
    n1.record.created_at = "2026-10-28T12:00:00Z".to_string();
    storage.save_note(&n1).unwrap();

    // Event 2: Oct 29 (15:00 UTC)
    let mut n2 = Note::new(project.id, "Day 2 Note", "Body", vec![], None).unwrap();
    n2.record.created_at = "2026-10-29T15:00:00Z".to_string();
    storage.save_note(&n2).unwrap();

    // Event 3: Oct 30 (18:00 UTC)
    let mut n3 = Note::new(project.id, "Day 3 Note", "Body", vec![], None).unwrap();
    n3.record.created_at = "2026-10-30T18:00:00Z".to_string();
    storage.save_note(&n3).unwrap();

    // Filter 1: Full day boundary "2026-10-29" to "2026-10-29"
    // Since until="2026-10-29" is inclusive of the full day (up to 23:59:59.999Z),
    // it MUST include Day 2 (15:00 UTC) and exclude Day 1 and Day 3.
    let filter_day2 = TimelineFilter {
        since: Some("2026-10-29".to_string()),
        until: Some("2026-10-29".to_string()),
        ..Default::default()
    };
    let events_day2 =
        TimelineService::assemble_timeline(&storage, project.id, root, &filter_day2).unwrap();
    assert_eq!(events_day2.len(), 1);
    assert_eq!(events_day2[0].title, "Day 2 Note");

    // Filter 2: Timestamp boundary since 2026-10-29T16:00:00Z
    // Excludes Day 2 (15:00 UTC), includes only Day 3
    let filter_since_late = TimelineFilter {
        since: Some("2026-10-29T16:00:00Z".to_string()),
        ..Default::default()
    };
    let events_late =
        TimelineService::assemble_timeline(&storage, project.id, root, &filter_since_late).unwrap();
    assert_eq!(events_late.len(), 1);
    assert_eq!(events_late[0].title, "Day 3 Note");

    // Filter 3: Timestamp boundary until 2026-10-29T14:00:00Z
    // Excludes Day 2 (15:00 UTC), includes only Day 1
    let filter_until_early = TimelineFilter {
        until: Some("2026-10-29T14:00:00Z".to_string()),
        ..Default::default()
    };
    let events_early =
        TimelineService::assemble_timeline(&storage, project.id, root, &filter_until_early)
            .unwrap();
    assert_eq!(events_early.len(), 1);
    assert_eq!(events_early[0].title, "Day 1 Note");
}

#[test]
fn test_bounded_retrieval_efficiency() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    init_git_repo(root);

    // Create 6 commits
    for i in 1..=6 {
        create_commit(
            root,
            &format!("bounded_{}.txt", i),
            "data",
            &format!("commit #{}", i),
        );
    }

    // Direct bounded query via GitAdapter
    let commits_3 = GitAdapter::get_commits(root, Some(3), None, None).unwrap();
    assert_eq!(commits_3.len(), 3);
    assert_eq!(commits_3[0].message, "commit #6");
    assert_eq!(commits_3[1].message, "commit #5");
    assert_eq!(commits_3[2].message, "commit #4");

    // Timeline limited retrieval
    let storage = SqliteStorage::open_in_memory().unwrap();
    let project = Project::new("Bounded Test", None).unwrap();
    storage.save_project(&project).unwrap();

    let filter_2 = TimelineFilter {
        limit: Some(2),
        ..Default::default()
    };
    let events = TimelineService::assemble_timeline(&storage, project.id, root, &filter_2).unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].title, "commit #6");
    assert_eq!(events[1].title, "commit #5");
    assert!(!events.is_truncated);
}

fn create_dated_commit(
    path: &Path,
    file_name: &str,
    content: &str,
    message: &str,
    date_str: &str,
) -> String {
    let file_path = path.join(file_name);
    let mut file = File::create(&file_path).unwrap();
    file.write_all(content.as_bytes()).unwrap();

    Command::new("git")
        .args(["add", file_name])
        .current_dir(path)
        .output()
        .expect("git add failed");

    Command::new("git")
        .env("GIT_AUTHOR_DATE", date_str)
        .env("GIT_COMMITTER_DATE", date_str)
        .args(["commit", "-m", message])
        .current_dir(path)
        .output()
        .expect("git commit failed");

    let out = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(path)
        .output()
        .expect("rev-parse failed");

    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn test_timeline_limit_and_date_filter_interaction() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    init_git_repo(root);

    let storage = SqliteStorage::open_in_memory().unwrap();
    let project = Project::new("Limit & Date Interaction Test", None).unwrap();
    storage.save_project(&project).unwrap();

    // 1. Create older commits in January 2026
    create_dated_commit(
        root,
        "jan1.txt",
        "1",
        "commit Jan 1",
        "2026-01-01T10:00:00Z",
    );
    create_dated_commit(
        root,
        "jan2.txt",
        "2",
        "commit Jan 2",
        "2026-01-02T10:00:00Z",
    );
    create_dated_commit(
        root,
        "jan3.txt",
        "3",
        "commit Jan 3",
        "2026-01-03T10:00:00Z",
    );

    // 2. Create newer commits in October 2026
    create_dated_commit(
        root,
        "oct30a.txt",
        "4",
        "commit Oct 30 A",
        "2026-10-30T10:00:00Z",
    );
    create_dated_commit(
        root,
        "oct30b.txt",
        "5",
        "commit Oct 30 B",
        "2026-10-30T12:00:00Z",
    );

    // 3. Create interleaved DevContext notes
    let mut note_jan = Note::new(project.id, "Note Jan 2", "Content", vec![], None).unwrap();
    note_jan.record.created_at = "2026-01-02T12:00:00Z".to_string();
    storage.save_note(&note_jan).unwrap();

    let mut note_oct = Note::new(project.id, "Note Oct 30", "Content", vec![], None).unwrap();
    note_oct.record.created_at = "2026-10-30T14:00:00Z".to_string();
    storage.save_note(&note_oct).unwrap();

    // -------------------------------------------------------------------------
    // Scenario A: Both --since and --until together with a limit
    // Newest commits (Oct 30 A & B) do not match, but older Jan commits and note do.
    // -------------------------------------------------------------------------
    let filter_jan_range = TimelineFilter {
        since: Some("2026-01-01T00:00:00Z".to_string()),
        until: Some("2026-01-03T23:59:59Z".to_string()),
        limit: Some(2),
        ..Default::default()
    };
    let events_jan =
        TimelineService::assemble_timeline(&storage, project.id, root, &filter_jan_range).unwrap();

    // Matching in date range: commit Jan 3 (Jan 3 10:00), note Jan 2 (Jan 2 12:00),
    // commit Jan 2 (Jan 2 10:00), commit Jan 1 (Jan 1 10:00).
    // With limit 2, must return the 2 newest matching events:
    assert_eq!(events_jan.len(), 2);
    assert_eq!(events_jan[0].title, "commit Jan 3");
    assert_eq!(events_jan[1].title, "Note Jan 2");
    assert!(!events_jan.is_truncated);

    // -------------------------------------------------------------------------
    // Scenario B: Only --until with a limit
    // Newest commits (Oct 30 A & B, Jan 3) are excluded by --until.
    // -------------------------------------------------------------------------
    let filter_until = TimelineFilter {
        until: Some("2026-01-02T23:59:59Z".to_string()),
        limit: Some(2),
        ..Default::default()
    };
    let events_until =
        TimelineService::assemble_timeline(&storage, project.id, root, &filter_until).unwrap();

    // Matching: Note Jan 2 (12:00), commit Jan 2 (10:00), commit Jan 1 (10:00).
    // With limit 2:
    assert_eq!(events_until.len(), 2);
    assert_eq!(events_until[0].title, "Note Jan 2");
    assert_eq!(events_until[1].title, "commit Jan 2");

    // -------------------------------------------------------------------------
    // Scenario C: Only --since with a limit
    // -------------------------------------------------------------------------
    let filter_since = TimelineFilter {
        since: Some("2026-10-30T00:00:00Z".to_string()),
        limit: Some(2),
        ..Default::default()
    };
    let events_since =
        TimelineService::assemble_timeline(&storage, project.id, root, &filter_since).unwrap();

    // Matching: Note Oct 30 (14:00), commit Oct 30 B (12:00), commit Oct 30 A (10:00).
    // With limit 2:
    assert_eq!(events_since.len(), 2);
    assert_eq!(events_since[0].title, "Note Oct 30");
    assert_eq!(events_since[1].title, "commit Oct 30 B");

    // -------------------------------------------------------------------------
    // Scenario D: Mixed Git and DevContext events where global limit is applied
    // strictly after merging all matching events.
    // -------------------------------------------------------------------------
    let filter_mixed = TimelineFilter {
        limit: Some(3),
        ..Default::default()
    };
    let events_mixed =
        TimelineService::assemble_timeline(&storage, project.id, root, &filter_mixed).unwrap();

    // Total events: 7 (5 commits + 2 notes).
    // Ordered descending:
    // 1. Note Oct 30 (14:00)
    // 2. commit Oct 30 B (12:00)
    // 3. commit Oct 30 A (10:00)
    // 4. commit Jan 3 (10:00)
    // ...
    // Global limit of 3 truncates to exactly the top 3:
    assert_eq!(events_mixed.len(), 3);
    assert_eq!(events_mixed[0].title, "Note Oct 30");
    assert_eq!(events_mixed[1].title, "commit Oct 30 B");
    assert_eq!(events_mixed[2].title, "commit Oct 30 A");
}
