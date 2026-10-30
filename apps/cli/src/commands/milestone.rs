use crate::args::MilestoneCommands;
use crate::output::OutputRenderer;
use devcontext_core::domain::{Milestone, MilestoneStatus};
use devcontext_core::error::{CoreError, Result};
use devcontext_core::workspace::WorkspaceManager;
use serde::Serialize;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Serialize)]
pub struct MilestoneSummary {
    pub id: String,
    pub title: String,
    pub status: String,
    pub target_date: Option<String>,
    pub completed_date: Option<String>,
}

impl From<&Milestone> for MilestoneSummary {
    fn from(m: &Milestone) -> Self {
        Self {
            id: m.record.id.to_string(),
            title: m.record.title.clone(),
            status: m.status.to_string(),
            target_date: m.target_date.clone(),
            completed_date: m.completed_date.clone(),
        }
    }
}

pub fn handle_milestone(
    project_dir: Option<PathBuf>,
    cmd: MilestoneCommands,
    renderer: &OutputRenderer,
) -> Result<i32> {
    let start_dir = match project_dir {
        Some(p) => p,
        None => std::env::current_dir()?,
    };

    let root = WorkspaceManager::find_project_root(&start_dir).ok_or_else(|| {
        CoreError::NotFound(format!(
            "No DevContext project found in '{}'. Run 'devcontext init' first.",
            start_dir.display()
        ))
    })?;

    let (project, storage) = WorkspaceManager::open_project(&root)?;

    match cmd {
        MilestoneCommands::Add {
            title,
            status,
            target_date,
            description,
            tag,
        } => {
            let parsed_status = MilestoneStatus::from_str(&status)?;
            let milestone = Milestone::new(
                project.id,
                title,
                parsed_status,
                target_date,
                description,
                tag,
            )?;

            storage.save_milestone(&milestone)?;

            let summary = MilestoneSummary::from(&milestone);
            renderer.success(&summary, || {
                format!(
                    "Added milestone [{}] \"{}\" [{}]",
                    summary.id, summary.title, summary.status
                )
            });
            Ok(0)
        }

        MilestoneCommands::List => {
            let milestones = storage.list_milestones(project.id)?;
            let summaries: Vec<MilestoneSummary> =
                milestones.iter().map(MilestoneSummary::from).collect();

            renderer.success(&summaries, || {
                if summaries.is_empty() {
                    return "No milestones found.".to_string();
                }
                let mut out = format!("Milestones ({}):\n", summaries.len());
                for m in &summaries {
                    let date_info = match (&m.target_date, &m.completed_date) {
                        (_, Some(c)) => format!(" (completed: {})", c),
                        (Some(t), None) => format!(" (target: {})", t),
                        (None, None) => String::new(),
                    };
                    out.push_str(&format!(
                        "- {} \"{}\" [{}]{}\n",
                        m.id,
                        m.title,
                        m.status.to_uppercase(),
                        date_info
                    ));
                }
                out
            });
            Ok(0)
        }

        MilestoneCommands::Complete { id } => {
            let milestone_id = uuid::Uuid::parse_str(&id).map_err(|e| {
                CoreError::Validation(format!("Invalid milestone UUID '{}': {}", id, e))
            })?;

            let mut milestone = storage
                .get_milestone(milestone_id)?
                .ok_or_else(|| CoreError::NotFound(format!("Milestone '{}' not found", id)))?;

            milestone.set_status(MilestoneStatus::Completed);
            storage.save_milestone(&milestone)?;

            let summary = MilestoneSummary::from(&milestone);
            renderer.success(&summary, || {
                format!(
                    "Completed milestone [{}] \"{}\" [{}]",
                    summary.id, summary.title, summary.status
                )
            });
            Ok(0)
        }
    }
}
