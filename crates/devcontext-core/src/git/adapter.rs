use super::model::GitCommitInfo;
use crate::error::Result;
use chrono::{DateTime, Utc};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Dedicated read-only Git adapter utilizing the system Git executable with strict offline guarantees.
pub struct GitAdapter;

impl GitAdapter {
    /// Creates a pre-configured read-only Git Command configured with offline invariants.
    fn git_command(dir: &Path) -> Command {
        Self::git_command_with_bin("git", dir)
    }

    /// Internal command builder allowing binary substitution (e.g. for missing executable testing).
    pub(crate) fn git_command_with_bin(bin: &str, dir: &Path) -> Command {
        let mut cmd = Command::new(bin);
        cmd.current_dir(dir);

        // Strict offline and execution guarantees:
        // 1. Prevent Git from performing lazy on-demand fetches in partial clones
        cmd.env("GIT_NO_LAZY_FETCH", "1");
        // 2. Prevent Git operations from acquiring optional background locks (e.g. index preload)
        cmd.env("GIT_OPTIONAL_LOCKS", "0");
        // 3. Never hang indefinitely on unexpected credential/passphrase terminal prompts
        cmd.env("GIT_TERMINAL_PROMPT", "0");

        cmd
    }

    /// Checks if a valid Git executable is available on the system PATH.
    pub fn is_git_installed() -> bool {
        Command::new("git")
            .env("GIT_TERMINAL_PROMPT", "0")
            .arg("--version")
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false)
    }

    /// Verifies if `path` is inside an active Git working tree.
    pub fn is_git_repository(path: impl AsRef<Path>) -> bool {
        let path = path.as_ref();
        let output = Self::git_command(path)
            .args(["rev-parse", "--is-inside-work-tree"])
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
        let path = path.as_ref();
        let output = Self::git_command(path)
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let raw_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if raw_path.is_empty() {
            None
        } else {
            Some(PathBuf::from(raw_path))
        }
    }

    /// Reads the name of the currently active branch or detects detached HEAD state.
    pub fn get_current_branch(path: impl AsRef<Path>) -> Result<Option<String>> {
        let path = path.as_ref();
        let output = match Self::git_command(path)
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
        {
            Ok(out) => out,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.into()),
        };

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

    /// Retrieves the full commit hash of HEAD.
    pub fn get_head_commit(path: impl AsRef<Path>) -> Result<Option<String>> {
        let path = path.as_ref();
        let output = match Self::git_command(path).args(["rev-parse", "HEAD"]).output() {
            Ok(out) => out,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.into()),
        };

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
        let path = path.as_ref();
        let target = format!("{}^{{commit}}", commit_hash.trim());
        Self::git_command(path)
            .args(["cat-file", "-e", &target])
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false)
    }

    /// Reads bounded commit history using ASCII record-separator (`\x1e`) and unit-separator (`\x1f`).
    ///
    /// If Git is not installed or the directory is not a Git repository, returns an empty Vec without crashing.
    pub fn get_commits(
        path: impl AsRef<Path>,
        limit: Option<usize>,
        since: Option<&str>,
        until: Option<&str>,
    ) -> Result<Vec<GitCommitInfo>> {
        Self::get_commits_with_bin("git", path, limit, since, until)
    }

    /// Internal method allowing custom executable testing (e.g. missing git binary simulation).
    #[doc(hidden)]
    pub fn get_commits_with_bin(
        bin: &str,
        path: impl AsRef<Path>,
        limit: Option<usize>,
        since: Option<&str>,
        until: Option<&str>,
    ) -> Result<Vec<GitCommitInfo>> {
        let path = path.as_ref();

        let mut args = vec![
            "log".to_string(),
            // Delimited: %x1e (record separator, ASCII 30) between commits and %x1f (unit separator, ASCII 31) between fields
            // %H (hash), %an (author name), %ae (author email), %aI (strict ISO-8601 date), %B (full commit message body)
            "--format=%x1e%H%x1f%an%x1f%ae%x1f%aI%x1f%B".to_string(),
        ];

        // Apply bounded retrieval directly to the Git subprocess to avoid unbounded memory loading
        if let Some(n) = limit {
            args.push(format!("-n{}", n));
        }

        if let Some(s) = since {
            args.push(format!("--since={}", s));
        }

        if let Some(u) = until {
            args.push(format!("--until={}", u));
        }

        let output = match Self::git_command_with_bin(bin, path).args(&args).output() {
            Ok(out) => out,
            Err(e) if e.kind() == ErrorKind::NotFound => {
                // Graceful degradation: Git binary not found
                return Ok(Vec::new());
            }
            Err(e) => return Err(e.into()),
        };

        if !output.status.success() {
            // Repositories with 0 commits or non-git folders return non-zero; handle gracefully
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut commits = Vec::new();

        // Split by record separator (ASCII 30, \x1e) to cleanly handle multiline messages
        for record in stdout.split('\x1e') {
            let trimmed = record.trim();
            if trimmed.is_empty() {
                continue;
            }

            let parts: Vec<&str> = trimmed.split('\x1f').collect();
            if parts.len() >= 5 {
                let hash = parts[0].trim();
                let author_name = parts[1].trim();
                let author_email = parts[2].trim();
                let raw_date = parts[3].trim();
                // Join in case the commit message itself contained unit separators
                let message = parts[4..].join("\x1f").trim().to_string();

                // Normalize date to UTC RFC 3339 if possible
                let normalized_date = match DateTime::parse_from_rfc3339(raw_date) {
                    Ok(dt) => dt.with_timezone(&Utc).to_rfc3339(),
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
