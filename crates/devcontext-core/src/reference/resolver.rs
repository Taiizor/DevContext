use crate::domain::{Reference, ReferenceType};
use crate::git::GitAdapter;
use crate::storage::SqliteStorage;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};
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

/// Result of safely validating a filesystem target against the project boundary.
#[derive(Debug, PartialEq, Eq)]
enum PathValidationResult {
    /// Target exists inside the project root at the given canonical path.
    Exists(PathBuf),
    /// Target does not exist, but its existing ancestor is safely inside the project root.
    TargetMissing,
    /// Path violates safety boundaries (traversal, external symlink/junction, unresolvable ancestor).
    Invalid(String),
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

            ReferenceType::File => match Self::validate_project_path(target, project_root) {
                PathValidationResult::Exists(canonical_path) => {
                    if canonical_path.is_file() {
                        ResolvedReference::resolved(reference.clone())
                    } else {
                        ResolvedReference::invalid(
                            reference.clone(),
                            "Target exists but is a directory, not a file",
                        )
                    }
                }
                PathValidationResult::TargetMissing => {
                    ResolvedReference::target_missing(reference.clone(), "File does not exist")
                }
                PathValidationResult::Invalid(reason) => {
                    ResolvedReference::invalid(reference.clone(), reason)
                }
            },

            ReferenceType::Folder => match Self::validate_project_path(target, project_root) {
                PathValidationResult::Exists(canonical_path) => {
                    if canonical_path.is_dir() {
                        ResolvedReference::resolved(reference.clone())
                    } else {
                        ResolvedReference::invalid(
                            reference.clone(),
                            "Target exists but is a file, not a directory",
                        )
                    }
                }
                PathValidationResult::TargetMissing => {
                    ResolvedReference::target_missing(reference.clone(), "Folder does not exist")
                }
                PathValidationResult::Invalid(reason) => {
                    ResolvedReference::invalid(reference.clone(), reason)
                }
            },

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

    /// Safely validates a file or directory reference against canonical project boundaries.
    /// Handles symlinks, junctions, reparse points, absolute/UNC paths, parent traversal,
    /// and nonexistent paths with ancestor verification.
    fn validate_project_path(target_str: &str, project_root: &Path) -> PathValidationResult {
        if target_str.is_empty() {
            return PathValidationResult::Invalid("Path cannot be empty".into());
        }

        let canonical_root = match project_root.canonicalize() {
            Ok(r) => r,
            Err(e) => {
                return PathValidationResult::Invalid(format!(
                    "Project root cannot be canonicalized: {}",
                    e
                ))
            }
        };

        let target_path = Path::new(target_str);
        let candidate = if target_path.is_absolute() {
            target_path.to_path_buf()
        } else {
            project_root.join(target_path)
        };

        // 1. Check if candidate itself is a symlink or junction
        if let Ok(meta) = candidate.symlink_metadata() {
            if meta.file_type().is_symlink() {
                // If the symlink/junction exists and points somewhere:
                match candidate.canonicalize() {
                    Ok(canonical_target) => {
                        if !Self::is_subpath(&canonical_target, &canonical_root) {
                            return PathValidationResult::Invalid(
                                "Symlink or junction target resolves outside project root".into(),
                            );
                        }
                        return PathValidationResult::Exists(canonical_target);
                    }
                    Err(_) => {
                        // Broken symlink/junction: inspect link destination
                        if let Ok(link_dest) = std::fs::read_link(&candidate) {
                            let full_dest = if link_dest.is_absolute() {
                                link_dest
                            } else {
                                candidate.parent().unwrap_or(project_root).join(link_dest)
                            };
                            let mut anc = full_dest.as_path();
                            while !anc.exists() {
                                match anc.parent() {
                                    Some(p) if p != anc => anc = p,
                                    _ => break,
                                }
                            }
                            if anc.exists() {
                                if let Ok(canonical_anc) = anc.canonicalize() {
                                    if !Self::is_subpath(&canonical_anc, &canonical_root) {
                                        return PathValidationResult::Invalid(
                                            "Broken symlink points outside project root".into(),
                                        );
                                    }
                                }
                            }
                        }
                        return PathValidationResult::TargetMissing;
                    }
                }
            }
        }

        // 2. Candidate exists on filesystem
        if candidate.exists() {
            match candidate.canonicalize() {
                Ok(canonical_target) => {
                    if Self::is_subpath(&canonical_target, &canonical_root) {
                        PathValidationResult::Exists(canonical_target)
                    } else {
                        PathValidationResult::Invalid("Path resolves outside project root".into())
                    }
                }
                Err(e) => {
                    PathValidationResult::Invalid(format!("Path cannot be canonicalized: {}", e))
                }
            }
        } else {
            // 3. Candidate does not exist. Check closest existing ancestor directory
            let mut existing_ancestor = candidate.as_path();
            let mut found_existing = false;

            while let Some(parent) = existing_ancestor.parent() {
                existing_ancestor = parent;
                if existing_ancestor.exists() {
                    found_existing = true;
                    break;
                }
                if existing_ancestor == parent {
                    break;
                }
            }

            if found_existing && existing_ancestor.exists() {
                match existing_ancestor.canonicalize() {
                    Ok(canonical_ancestor) => {
                        if Self::is_subpath(&canonical_ancestor, &canonical_root) {
                            PathValidationResult::TargetMissing
                        } else {
                            PathValidationResult::Invalid(
                                "Path ancestor resolves outside project root".into(),
                            )
                        }
                    }
                    Err(e) => PathValidationResult::Invalid(format!(
                        "Path ancestor cannot be canonicalized: {}",
                        e
                    )),
                }
            } else {
                PathValidationResult::Invalid(
                    "Path does not have an accessible ancestor inside project root".into(),
                )
            }
        }
    }

    /// Determines if `target` resides within or is equal to `root`, platform-aware.
    fn is_subpath(target: &Path, root: &Path) -> bool {
        #[cfg(windows)]
        {
            let target_str = target.to_string_lossy().to_lowercase().replace('/', "\\");
            let root_str = root.to_string_lossy().to_lowercase().replace('/', "\\");

            if target_str == root_str {
                return true;
            }
            let prefix = if root_str.ends_with('\\') {
                root_str
            } else {
                format!("{}\\", root_str)
            };
            target_str.starts_with(&prefix)
        }
        #[cfg(not(windows))]
        {
            target.starts_with(root)
        }
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
