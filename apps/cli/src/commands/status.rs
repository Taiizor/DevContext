use crate::output::OutputRenderer;
use devcontext_core::error::{CoreError, Result};
use devcontext_core::workspace::WorkspaceManager;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
pub struct StatusOutput {
    pub project_id: String,
    pub name: String,
    pub root_path: String,
    pub created_at: String,
    pub vcs_identifier: Option<String>,
    pub stats: ProjectStats,
}

#[derive(Serialize)]
pub struct ProjectStats {
    pub notes_count: usize,
    pub decisions_count: usize,
    pub milestones_count: usize,
    pub log_entries_count: usize,
}

pub fn handle_status(project_dir: Option<PathBuf>, renderer: &OutputRenderer) -> Result<i32> {
    let start_dir = match project_dir {
        Some(p) => p,
        None => std::env::current_dir()?,
    };

    let root = WorkspaceManager::find_project_root(&start_dir).ok_or_else(|| {
        CoreError::NotFound(format!(
            "No DevContext project found in '{}' or parent directories. Run 'devcontext init' to create one.",
            start_dir.display()
        ))
    })?;

    let (project, storage) = WorkspaceManager::open_project(&root)?;

    let notes = storage.list_notes(project.id, true)?;
    let decisions = storage.list_decisions(project.id)?;
    let milestones = storage.list_milestones(project.id)?;
    let logs = storage.list_log_entries(project.id)?;

    let stats = ProjectStats {
        notes_count: notes.len(),
        decisions_count: decisions.len(),
        milestones_count: milestones.len(),
        log_entries_count: logs.len(),
    };

    let output = StatusOutput {
        project_id: project.id.to_string(),
        name: project.name.clone(),
        root_path: root.display().to_string(),
        created_at: project.created_at.clone(),
        vcs_identifier: project.vcs_identifier.clone(),
        stats,
    };

    renderer.success(&output, || {
        let mut text = format!(
            "Project: {} ({})\nLocation: {}\nCreated: {}\n",
            output.name, output.project_id, output.root_path, output.created_at
        );
        if let Some(vcs) = &output.vcs_identifier {
            text.push_str(&format!("VCS HEAD: {}\n", vcs));
        }
        text.push_str(&format!(
            "\nStored Knowledge:\n  Notes: {}\n  Decisions: {}\n  Milestones: {}\n  Log Entries: {}",
            output.stats.notes_count,
            output.stats.decisions_count,
            output.stats.milestones_count,
            output.stats.log_entries_count
        ));
        text
    });

    Ok(0)
}
