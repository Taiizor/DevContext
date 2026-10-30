use crate::domain::{Reference, ReferenceType};
use crate::git::GitAdapter;
use crate::storage::SqliteStorage;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Component, Path};
use uuid::Uuid;

/// Status indicating the resolution result of a reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceResolutionStatus {
    /// The target exists, is accessible, and verified.
    Resolved,
    /// The reference syntax is valid, but the target does not exist locally (e.g. deleted file or commit).
    TargetMissing,
    /// The reference target is syntactically invalid or violates safety boundaries (e.g. path traversal).
    Invalid,
    /// Target state cannot be determined (e.g. Git repository not present for a Git reference).
    Unavailable,
}

impl fmt::Display for ReferenceResolutionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Resolved => write!(f, "resolved"),
            Self::TargetMissing => write!(f, "target_missing"),
            Self::Invalid => write!(f, "invalid"),
            Self::Unavailable => write!(f, "unavailable"),
        }
    }
}

/// A reference accompanied by its resolution status and diagnostic message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResolvedReference {
    pub reference: Reference,
    pub status: ReferenceResolutionStatus,
    pub message: Option<String>,
}

impl ResolvedReference {
    pub fn new(
        reference: Reference,
        status: ReferenceResolutionStatus,
        message: Option<String>,
    ) -> Self {
        Self {
            reference,
            status,
            message,
        }
    }

    pub fn resolved(reference: Reference) -> Self {
        Self {
            reference,
            status: ReferenceResolutionStatus::Resolved,
            message: None,
        }
    }

    pub fn target_missing(reference: Reference, message: impl Into<String>) -> Self {
        Self {
            reference,
            status: ReferenceResolutionStatus::TargetMissing,
            message: Some(message.into()),
        }
    }

    pub fn invalid(reference: Reference, message: impl Into<String>) -> Self {
        Self {
            reference,
            status: ReferenceResolutionStatus::Invalid,
            message: Some(message.into()),
        }
    }

    pub fn unavailable(reference: Reference, message: impl Into<String>) -> Self {
        Self {
            reference,
            status: ReferenceResolutionStatus::Unavailable,
            message: Some(message.into()),
        }
    }
}

/// Resolves DevContext references safely against the local filesystem, Git, and database.
pub struct ReferenceResolver;

impl ReferenceResolver {
    /// Resolves a single reference against the project environment.
    pub fn resolve(
        reference: &Reference,
        project_root: &Path,
        storage: Option<&SqliteStorage>,
    ) -> ResolvedReference {
        let target = reference.target.trim();

        match reference.ref_type {
            ReferenceType::GitCommit => {
                // Must be a valid 7 to 64 character hex string
                if target.len() < 7
                    || target.len() > 64
                    || !target.chars().all(|c| c.is_ascii_hexdigit())
                {
                    return ResolvedReference::invalid(
                        reference.clone(),
                        "Git commit hash must be 7 to 64 hexadecimal characters",
                    );
                }

                if !GitAdapter::is_git_repository(project_root) {
                    return ResolvedReference::unavailable(
                        reference.clone(),
                        "Git repository is not available in workspace",
                    );
                }

                if GitAdapter::commit_exists(project_root, target) {
                    ResolvedReference::resolved(reference.clone())
                } else {
                    ResolvedReference::target_missing(
                        reference.clone(),
                        format!("Commit '{}' not found in local repository", target),
                    )
                }
            }

            ReferenceType::GitBranch => {
                if target.is_empty() {
                    return ResolvedReference::invalid(
                        reference.clone(),
                        "Git branch name cannot be empty",
                    );
                }

                if !GitAdapter::is_git_repository(project_root) {
                    return ResolvedReference::unavailable(
                        reference.clone(),
                        "Git repository is not available in workspace",
                    );
                }

                // Verify branch exists locally using git rev-parse --verify
                let output = GitAdapter::git_command_with_bin("git", project_root)
                    .args(["rev-parse", "--verify", &format!("refs/heads/{}", target)])
                    .output();

                match output {
                    Ok(out) if out.status.success() => {
                        ResolvedReference::resolved(reference.clone())
                    }
                    _ => ResolvedReference::target_missing(
                        reference.clone(),
                        format!("Branch '{}' does not exist in local repository", target),
                    ),
                }
            }

            ReferenceType::File => {
                if target.is_empty() {
                    return ResolvedReference::invalid(
                        reference.clone(),
                        "File path cannot be empty",
                    );
                }

                let target_path = Path::new(target);

                // Security: Prevent path traversal escaping project root
                if Self::is_traversal_outside_root(target_path, project_root) {
                    return ResolvedReference::invalid(
                        reference.clone(),
                        "File reference traverses outside project root",
                    );
                }

                let full_path = if target_path.is_absolute() {
                    target_path.to_path_buf()
                } else {
                    project_root.join(target_path)
                };

                if full_path.exists() {
                    if full_path.is_file() {
                        ResolvedReference::resolved(reference.clone())
                    } else {
                        ResolvedReference::invalid(
                            reference.clone(),
                            "Target exists but is a directory, not a file",
                        )
                    }
                } else {
                    ResolvedReference::target_missing(reference.clone(), "File does not exist")
                }
            }

            ReferenceType::Folder => {
                if target.is_empty() {
                    return ResolvedReference::invalid(
                        reference.clone(),
                        "Folder path cannot be empty",
                    );
                }

                let target_path = Path::new(target);

                // Security: Prevent path traversal escaping project root
                if Self::is_traversal_outside_root(target_path, project_root) {
                    return ResolvedReference::invalid(
                        reference.clone(),
                        "Folder reference traverses outside project root",
                    );
                }

                let full_path = if target_path.is_absolute() {
                    target_path.to_path_buf()
                } else {
                    project_root.join(target_path)
                };

                if full_path.exists() {
                    if full_path.is_dir() {
                        ResolvedReference::resolved(reference.clone())
                    } else {
                        ResolvedReference::invalid(
                            reference.clone(),
                            "Target exists but is a file, not a directory",
                        )
                    }
                } else {
                    ResolvedReference::target_missing(reference.clone(), "Folder does not exist")
                }
            }

            ReferenceType::Url => {
                // Strict syntactical validation without any network access
                if Self::is_valid_url_syntax(target) {
                    ResolvedReference::resolved(reference.clone())
                } else {
                    ResolvedReference::invalid(reference.clone(), "Malformed URL syntax")
                }
            }

            ReferenceType::Symbol => {
                if target.is_empty() {
                    ResolvedReference::invalid(reference.clone(), "Symbol target cannot be empty")
                } else {
                    ResolvedReference::resolved(reference.clone())
                }
            }

            ReferenceType::Command => {
                if target.is_empty() {
                    ResolvedReference::invalid(reference.clone(), "Command target cannot be empty")
                } else {
                    ResolvedReference::resolved(reference.clone())
                }
            }

            ReferenceType::Record => {
                let Ok(record_id) = Uuid::parse_str(target) else {
                    return ResolvedReference::invalid(
                        reference.clone(),
                        "Record reference target is not a valid UUID",
                    );
                };

                if let Some(storage) = storage {
                    // Check if target record exists and is not deleted
                    match storage.get_record_envelope(record_id) {
                        Ok(Some(rec)) if !rec.is_deleted => {
                            ResolvedReference::resolved(reference.clone())
                        }
                        _ => ResolvedReference::target_missing(
                            reference.clone(),
                            "Referenced record does not exist or has been deleted",
                        ),
                    }
                } else {
                    ResolvedReference::resolved(reference.clone())
                }
            }
        }
    }

    /// Resolves an array of references against the project environment.
    pub fn resolve_all(
        references: &[Reference],
        project_root: &Path,
        storage: Option<&SqliteStorage>,
    ) -> Vec<ResolvedReference> {
        references
            .iter()
            .map(|r| Self::resolve(r, project_root, storage))
            .collect()
    }

    /// Checks whether `path` traverses or points outside `root`.
    fn is_traversal_outside_root(path: &Path, root: &Path) -> bool {
        if path.is_absolute() {
            return !path.starts_with(root);
        }

        let mut depth: isize = 0;
        for comp in path.components() {
            match comp {
                Component::ParentDir => {
                    depth -= 1;
                    if depth < 0 {
                        return true;
                    }
                }
                Component::Normal(_) => {
                    depth += 1;
                }
                Component::RootDir | Component::Prefix(_) => {
                    return true;
                }
                _ => {}
            }
        }

        false
    }

    /// Syntactically validates a URL without network calls.
    fn is_valid_url_syntax(s: &str) -> bool {
        let lower = s.to_lowercase();
        // Accepted schemes
        let has_valid_scheme = lower.starts_with("http://")
            || lower.starts_with("https://")
            || lower.starts_with("git://")
            || lower.starts_with("ssh://")
            || lower.starts_with("file://")
            || lower.starts_with("ftp://");

        if !has_valid_scheme {
            return false;
        }

        let Some(rest) = s.split("://").nth(1) else {
            return false;
        };

        if rest.trim().is_empty() {
            return false;
        }

        // Host part before slash or query must not be empty
        let host_part = rest.split(['/', '?', '#']).next().unwrap_or("");
        if host_part.is_empty() {
            return false;
        }

        // Disallow spaces or control characters in URL
        !s.chars().any(|c| c.is_whitespace() || c.is_control())
    }
}
