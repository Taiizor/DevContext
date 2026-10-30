use crate::output::OutputRenderer;
use devcontext_core::error::Result;
use devcontext_core::workspace::WorkspaceManager;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
pub struct InitOutput {
    pub project_id: String,
    pub name: String,
    pub path: String,
    pub git_exclude_configured: bool,
}

pub fn handle_init(
    project_dir: Option<PathBuf>,
    name: Option<String>,
    renderer: &OutputRenderer,
) -> Result<i32> {
    let target_dir = match project_dir {
        Some(p) => p,
        None => std::env::current_dir()?,
    };

    let (project, _) = WorkspaceManager::initialize_project(&target_dir, name.as_deref())?;
    let git_excluded = WorkspaceManager::ensure_git_exclude(&target_dir).unwrap_or(false);

    let output = InitOutput {
        project_id: project.id.to_string(),
        name: project.name.clone(),
        path: target_dir.display().to_string(),
        git_exclude_configured: git_excluded,
    };

    renderer.success(&output, || {
        format!(
            "Initialized empty DevContext project \"{}\" ({})\nStorage: {}/.devcontext/devcontext.db{}",
            output.name,
            output.project_id,
            output.path,
            if git_excluded {
                "\nConfigured local Git exclude (.git/info/exclude)."
            } else {
                ""
            }
        )
    });

    Ok(0)
}
