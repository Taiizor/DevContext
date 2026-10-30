use crate::domain::Project;
use crate::error::{CoreError, Result};
use crate::storage::{SqliteStorage, DEFAULT_DB_FILENAME, DEFAULT_STORAGE_DIR};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Handles project discovery, workspace roots, and initialization.
pub struct WorkspaceManager;

impl WorkspaceManager {
    /// Walks upwards from `start_dir` to locate the nearest `.devcontext/` directory.
    pub fn find_project_root(start_dir: impl AsRef<Path>) -> Option<PathBuf> {
        let mut current = start_dir.as_ref().to_path_buf();

        loop {
            let devcontext_dir = current.join(DEFAULT_STORAGE_DIR);
            let db_file = devcontext_dir.join(DEFAULT_DB_FILENAME);

            if devcontext_dir.is_dir() && db_file.is_file() {
                return Some(current);
            }

            if !current.pop() {
                break;
            }
        }

        None
    }

    /// Walks upwards from `start_dir` to locate the nearest `.git` directory.
    pub fn find_git_root(start_dir: impl AsRef<Path>) -> Option<PathBuf> {
        let mut current = start_dir.as_ref().to_path_buf();

        loop {
            let git_dir = current.join(".git");
            if git_dir.is_dir() || git_dir.is_file() {
                return Some(current);
            }

            if !current.pop() {
                break;
            }
        }

        None
    }

    /// Initializes a new DevContext project in the given `project_root`.
    ///
    /// Fails with `CoreError::Project` if the project is already initialized.
    pub fn initialize_project(
        project_root: impl AsRef<Path>,
        name: Option<&str>,
    ) -> Result<(Project, SqliteStorage)> {
        let root = project_root.as_ref();
        let devcontext_dir = root.join(DEFAULT_STORAGE_DIR);
        let db_path = devcontext_dir.join(DEFAULT_DB_FILENAME);

        if db_path.exists() {
            return Err(CoreError::Project(format!(
                "Project is already initialized at: {}",
                root.display()
            )));
        }

        fs::create_dir_all(&devcontext_dir)?;

        let storage = SqliteStorage::open(&db_path)?;

        let project_name = match name {
            Some(n) if !n.trim().is_empty() => n.trim().to_string(),
            _ => root
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("unnamed-project")
                .to_string(),
        };

        // Determine optional VCS fingerprint if Git is present
        let vcs_identifier = Self::detect_vcs_fingerprint(root);

        let project = Project::new(project_name, vcs_identifier)?;
        storage.save_project(&project)?;

        // Ensure .devcontext/ is excluded in .git/info/exclude per ADR 0003
        let _ = Self::ensure_git_exclude(root);

        Ok((project, storage))
    }

    /// Opens an already initialized DevContext project.
    pub fn open_project(project_root: impl AsRef<Path>) -> Result<(Project, SqliteStorage)> {
        let root = project_root.as_ref();
        let db_path = root.join(DEFAULT_STORAGE_DIR).join(DEFAULT_DB_FILENAME);

        if !db_path.exists() {
            return Err(CoreError::NotFound(format!(
                "No DevContext project found at '{}'. Run 'devcontext init' first.",
                root.display()
            )));
        }

        let storage = SqliteStorage::open(&db_path)?;
        let projects = storage.list_projects()?;

        let project = projects.into_iter().next().ok_or_else(|| {
            CoreError::Integrity("Database initialized but no project record found".to_string())
        })?;

        Ok((project, storage))
    }

    /// Appends `.devcontext/` to `.git/info/exclude` if inside a Git repository per ADR 0003.
    pub fn ensure_git_exclude(project_root: impl AsRef<Path>) -> Result<bool> {
        let root = project_root.as_ref();
        let git_dir = root.join(".git");

        if !git_dir.is_dir() {
            return Ok(false);
        }

        let info_dir = git_dir.join("info");
        let exclude_file = info_dir.join("exclude");

        fs::create_dir_all(&info_dir)?;

        if exclude_file.exists() {
            let content = fs::read_to_string(&exclude_file)?;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed == ".devcontext"
                    || trimmed == ".devcontext/"
                    || trimmed == "/.devcontext/"
                {
                    return Ok(false); // Already excluded
                }
            }
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&exclude_file)?;

        writeln!(
            file,
            "\n# DevContext local project memory (added by devcontext init)\n.devcontext/"
        )?;

        Ok(true)
    }

    /// Basic VCS root identification (non-sensitive).
    fn detect_vcs_fingerprint(root: &Path) -> Option<String> {
        let head_file = root.join(".git").join("HEAD");
        if head_file.exists() {
            fs::read_to_string(head_file)
                .ok()
                .map(|s| s.trim().to_string())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_initialize_and_open_project() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        // 1. Initializing in fresh directory
        let (project, storage) =
            WorkspaceManager::initialize_project(root, Some("TestApp")).unwrap();
        assert_eq!(project.name, "TestApp");

        // 2. Initializing again in same directory must fail
        let re_init = WorkspaceManager::initialize_project(root, Some("DuplicateApp"));
        assert!(re_init.is_err());

        // 3. Opening from root
        let (opened, _) = WorkspaceManager::open_project(root).unwrap();
        assert_eq!(opened.id, project.id);
        assert_eq!(opened.name, "TestApp");

        // 4. Discovering from nested directory
        let nested = root.join("src").join("controllers");
        fs::create_dir_all(&nested).unwrap();

        let discovered = WorkspaceManager::find_project_root(&nested).unwrap();
        assert_eq!(discovered, root);
        drop(storage);
    }

    #[test]
    fn test_git_exclude_configuration() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        // Mock a git directory
        let git_dir = root.join(".git");
        fs::create_dir_all(&git_dir).unwrap();

        // Initialize DevContext
        WorkspaceManager::initialize_project(root, Some("GitRepo")).unwrap();

        let exclude_file = git_dir.join("info").join("exclude");
        assert!(exclude_file.exists());
        let content = fs::read_to_string(&exclude_file).unwrap();
        assert!(content.contains(".devcontext/"));

        // Running again shouldn't duplicate
        let added_again = WorkspaceManager::ensure_git_exclude(root).unwrap();
        assert!(!added_again);
    }
}
