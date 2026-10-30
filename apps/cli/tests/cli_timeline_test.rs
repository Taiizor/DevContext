#![allow(clippy::needless_borrows_for_generic_args)]

use devcontext_cli::run_from_iter;
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
        .args(["config", "user.name", "CLI Timeline Tester"])
        .current_dir(path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["config", "user.email", "clitimeline@devcontext.local"])
        .current_dir(path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["config", "commit.gpgsign", "false"])
        .current_dir(path)
        .output()
        .unwrap();
}

fn create_commit(path: &Path, file_name: &str, content: &str, message: &str) {
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
}

#[test]
fn test_cli_timeline_command_workflows() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let root_str = root.to_str().unwrap();

    // 1. Initialize Git & DevContext
    init_git_repo(root);
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "init",
            "--name",
            "TimelineApp"
        ]),
        0
    );

    // 2. Add git commit
    create_commit(root, "main.rs", "fn main() {}", "feat: initial entrypoint");

    // 3. Add note
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "note",
            "add",
            "Initial Design Note",
            "--content",
            "Context behind main.rs"
        ]),
        0
    );

    // 4. Add ADR
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "decision",
            "add",
            "ADR 0001: Architecture",
            "--context",
            "Context",
            "--decision",
            "Decision",
            "--consequences",
            "Consequences"
        ]),
        0
    );

    // 5. Run devcontext timeline (human readable)
    assert_eq!(
        run_from_iter(&["devcontext", "--project-dir", root_str, "timeline"]),
        0
    );

    // 6. Run devcontext timeline --json
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--json",
            "--project-dir",
            root_str,
            "timeline"
        ]),
        0
    );

    // 7. Run devcontext timeline --type commit
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "timeline",
            "--type",
            "commit"
        ]),
        0
    );

    // 8. Run devcontext timeline --type note
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "timeline",
            "--type",
            "note"
        ]),
        0
    );

    // 9. Run devcontext timeline --limit 1
    assert_eq!(
        run_from_iter(&[
            "devcontext",
            "--project-dir",
            root_str,
            "timeline",
            "--limit",
            "1"
        ]),
        0
    );
}
