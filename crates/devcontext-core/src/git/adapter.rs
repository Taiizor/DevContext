use super::model::GitCommitInfo;
use crate::error::Result;
use chrono::{DateTime, Utc};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Dedicated read-only Git adapter utilizing the system Git executable.
pub struct GitAdapter;

impl GitAdapter {
    /// Checks if a valid Git executable is available on the system PATH.
    pub fn is_git_installed() -> bool {
        Command::new("git")
            .arg("--version")
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false)
    }

    /// Verifies if `path` is inside an active Git working tree.
    pub fn is_git_repository(path: impl AsRef<Path>) -> bool {
        let output = Command::new("git")
            .args(["rev-parse", "--is-inside-work-tree"])
            .current_dir(path)
            .output();

        match output {
            Ok(out) => {
                out.status.success() && String::from_utf8_lossy(&out.stdout).trim() == "true"
            }
            Err(_) => false,
        }
    }

    /// Discovers the absolute root directory of the Git repository.
    pub fn get_repo_root(path: impl AsRef<Path>) -> Option<PathBuf> {
        let output = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .current_dir(path)
            .output()
            .ok()?;

        if output.status.success() {
            let root_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !root_str.is_empty() {
                return Some(PathBuf::from(root_str));
            }
        }
        None
    }

    /// Reads the name of the currently active branch or detects detached HEAD state.
    pub fn get_current_branch(path: impl AsRef<Path>) -> Result<Option<String>> {
        let output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(path)
            .output()?;

        if !output.status.success() {
            return Ok(None);
        }

        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if branch.is_empty() {
            Ok(None)
        } else if branch == "HEAD" {
            Ok(Some("HEAD (detached)".to_string()))
        } else {
            Ok(Some(branch))
        }
    }

    /// Retrieves the full 40-character commit hash of HEAD.
    pub fn get_head_commit(path: impl AsRef<Path>) -> Result<Option<String>> {
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(path)
            .output()?;

        if !output.status.success() {
            return Ok(None);
        }

        let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if hash.is_empty() {
            Ok(None)
        } else {
            Ok(Some(hash))
        }
    }

    /// Checks whether a commit with the specified hash exists locally in the repository.
    pub fn commit_exists(path: impl AsRef<Path>, commit_hash: &str) -> bool {
        let target = format!("{}^{{commit}}", commit_hash.trim());
        Command::new("git")
            .args(["cat-file", "-e", &target])
            .current_dir(path)
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false)
    }

    /// Reads bounded commit history using unit-separator (`\x1f`) formatted output.
    pub fn get_commits(
        path: impl AsRef<Path>,
        limit: Option<usize>,
        since: Option<&str>,
        until: Option<&str>,
    ) -> Result<Vec<GitCommitInfo>> {
        let path = path.as_ref();
        if !Self::is_git_repository(path) {
            return Ok(Vec::new());
        }

        let mut args = vec![
            "log".to_string(),
            // Unit-separator delimited: %H (hash), %an (author), %ae (email), %aI (strict ISO-8601), %s (subject)
            "--format=%H\x1f%an\x1f%ae\x1f%aI\x1f%s".to_string(),
        ];

        if let Some(n) = limit {
            args.push(format!("-n{}", n));
        }
        if let Some(s) = since {
            args.push(format!("--since={}", s));
        }
        if let Some(u) = until {
            args.push(format!("--until={}", u));
        }

        let output = Command::new("git").args(&args).current_dir(path).output()?;

        if !output.status.success() {
            // Repositories with 0 commits will return non-zero on git log; handle gracefully
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut commits = Vec::new();

        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let parts: Vec<&str> = trimmed.split('\x1f').collect();
            if parts.len() >= 5 {
                let hash = parts[0];
                let author_name = parts[1];
                let author_email = parts[2];
                let raw_date = parts[3];
                let message = parts[4];

                // Normalize date to UTC RFC 3339
                let normalized_date = match DateTime::parse_from_rfc3339(raw_date) {
                    Ok(parsed) => parsed.with_timezone(&Utc).to_rfc3339(),
                    Err(_) => raw_date.to_string(),
                };

                commits.push(GitCommitInfo::new(
                    hash,
                    author_name,
                    author_email,
                    normalized_date,
                    message,
                ));
            }
        }

        Ok(commits)
    }
}
