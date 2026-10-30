use devcontext_core::git::GitAdapter;
use std::fs::{self, File};
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
        .args(["config", "user.name", "DevContext Tester"])
        .current_dir(path)
        .output()
        .expect("git config user.name failed");

    Command::new("git")
        .args(["config", "user.email", "tester@devcontext.local"])
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
fn test_git_adapter_empty_repository_and_basics() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // 1. Non-git directory
    assert!(!GitAdapter::is_git_repository(root));
    assert_eq!(GitAdapter::get_repo_root(root), None);
    assert_eq!(
        GitAdapter::get_commits(root, None, None, None)
            .unwrap()
            .len(),
        0
    );

    // 2. Initialize Git
    init_git_repo(root);
    assert!(GitAdapter::is_git_repository(root));

    // 3. Empty Git repository (0 commits)
    let commits = GitAdapter::get_commits(root, None, None, None).unwrap();
    assert_eq!(commits.len(), 0);
    assert_eq!(GitAdapter::get_head_commit(root).unwrap(), None);

    // 4. Create first commit
    let c1_hash = create_commit(root, "file1.txt", "v1", "feat: initial commit");
    assert!(!c1_hash.is_empty());
    assert_eq!(
        GitAdapter::get_head_commit(root).unwrap(),
        Some(c1_hash.clone())
    );
    assert!(GitAdapter::commit_exists(root, &c1_hash));
    assert!(!GitAdapter::commit_exists(
        root,
        "0000000000000000000000000000000000000000"
    ));

    // 5. Read commits
    let commits_after = GitAdapter::get_commits(root, None, None, None).unwrap();
    assert_eq!(commits_after.len(), 1);
    assert_eq!(commits_after[0].hash, c1_hash);
    assert_eq!(commits_after[0].author_name, "DevContext Tester");
    assert_eq!(commits_after[0].message, "feat: initial commit");
}

#[test]
fn test_git_adapter_branches_and_detached_head() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    init_git_repo(root);

    let c1 = create_commit(root, "base.txt", "base", "base commit");

    // Default branch detection
    let initial_branch = GitAdapter::get_current_branch(root).unwrap().unwrap();
    assert!(!initial_branch.is_empty());

    // Create & checkout new branch
    Command::new("git")
        .args(["checkout", "-b", "feature/timeline-adapter"])
        .current_dir(root)
        .output()
        .unwrap();

    let feat_branch = GitAdapter::get_current_branch(root).unwrap().unwrap();
    assert_eq!(feat_branch, "feature/timeline-adapter");

    // Detach HEAD
    Command::new("git")
        .args(["checkout", &c1])
        .current_dir(root)
        .output()
        .unwrap();

    let detached = GitAdapter::get_current_branch(root).unwrap().unwrap();
    assert_eq!(detached, "HEAD (detached)");
}

#[test]
fn test_git_adapter_strict_read_only_behavior() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    init_git_repo(root);
    create_commit(root, "data.txt", "immutable", "Initial commit");

    // Create an unstaged dirty file
    let dirty_file = root.join("untracked.txt");
    fs::write(&dirty_file, "user local edit").unwrap();

    // Verify git status shows untracked file
    let status_before = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(root)
        .output()
        .unwrap();
    let before_str = String::from_utf8_lossy(&status_before.stdout);
    assert!(before_str.contains("untracked.txt"));

    // Execute multiple GitAdapter queries
    let _ = GitAdapter::is_git_repository(root);
    let _ = GitAdapter::get_current_branch(root);
    let _ = GitAdapter::get_head_commit(root);
    let _ = GitAdapter::get_commits(root, Some(10), None, None);

    // Verify status is completely untouched
    let status_after = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(root)
        .output()
        .unwrap();
    let after_str = String::from_utf8_lossy(&status_after.stdout);
    assert_eq!(before_str, after_str);
}
