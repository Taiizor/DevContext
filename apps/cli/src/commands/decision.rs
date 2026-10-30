use crate::args::DecisionCommands;
use crate::output::OutputRenderer;
use devcontext_core::domain::{Decision, DecisionStatus};
use devcontext_core::error::{CoreError, Result};
use devcontext_core::workspace::WorkspaceManager;
use serde::Serialize;
use std::path::PathBuf;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Serialize)]
pub struct DecisionSummary {
    pub id: String,
    pub title: String,
    pub status: String,
    pub superseded_by: Option<String>,
    pub created_at: String,
    pub tags: Vec<String>,
}

impl From<&Decision> for DecisionSummary {
    fn from(d: &Decision) -> Self {
        Self {
            id: d.record.id.to_string(),
            title: d.record.title.clone(),
            status: d.status.to_string(),
            superseded_by: d.superseded_by.map(|u| u.to_string()),
            created_at: d.record.created_at.clone(),
            tags: d.record.tags.clone(),
        }
    }
}

pub fn handle_decision(
    project_dir: Option<PathBuf>,
    cmd: DecisionCommands,
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
        DecisionCommands::Add {
            title,
            context,
            decision,
            consequences,
            alternatives,
            status,
            tag,
        } => {
            let parsed_status = DecisionStatus::from_str(&status)?;
            let record = Decision::new(
                project.id,
                title,
                parsed_status,
                context,
                decision,
                consequences,
                alternatives,
                tag,
            )?;

            storage.save_decision(&record)?;

            let summary = DecisionSummary::from(&record);
            renderer.success(&summary, || {
                format!(
                    "Recorded decision [{}] \"{}\" ({})",
                    summary.id, summary.title, summary.status
                )
            });
            Ok(0)
        }

        DecisionCommands::List => {
            let decisions = storage.list_decisions(project.id)?;
            let summaries: Vec<DecisionSummary> =
                decisions.iter().map(DecisionSummary::from).collect();

            renderer.success(&summaries, || {
                if summaries.is_empty() {
                    return "No decisions recorded.".to_string();
                }
                let mut out = format!("Architecture Decisions ({}):\n", summaries.len());
                for d in &summaries {
                    let superseded_info = match &d.superseded_by {
                        Some(rep) => format!(" -> Superseded by [{}]", rep),
                        None => String::new(),
                    };
                    out.push_str(&format!(
                        "- {} \"{}\" [{}]{}\n",
                        d.id,
                        d.title,
                        d.status.to_uppercase(),
                        superseded_info
                    ));
                }
                out
            });
            Ok(0)
        }

        DecisionCommands::Show { id } => {
            let dec_id = Uuid::parse_str(&id).map_err(|e| {
                CoreError::Validation(format!("Invalid decision UUID '{}': {}", id, e))
            })?;

            let decision = storage.get_decision(dec_id)?.ok_or_else(|| {
                CoreError::NotFound(format!("Decision with id '{}' not found", id))
            })?;

            renderer.success(&decision, || {
                let mut out = format!(
                    "# {}\nID: {}\nStatus: {}\nCreated: {}\n",
                    decision.record.title,
                    decision.record.id,
                    decision.status.to_string().to_uppercase(),
                    decision.record.created_at
                );

                if let Some(rep) = decision.superseded_by {
                    out.push_str(&format!("Superseded By: {}\n", rep));
                }

                out.push_str(&format!(
                    "\n## Context\n{}\n\n## Decision\n{}\n\n## Consequences\n{}",
                    decision.context, decision.decision, decision.consequences
                ));

                if let Some(alts) = &decision.alternatives_considered {
                    out.push_str(&format!("\n\n## Alternatives Considered\n{}", alts));
                }

                out
            });
            Ok(0)
        }

        DecisionCommands::Update {
            id,
            status,
            superseded_by,
        } => {
            let dec_id = Uuid::parse_str(&id).map_err(|e| {
                CoreError::Validation(format!("Invalid decision UUID '{}': {}", id, e))
            })?;

            let mut decision = storage.get_decision(dec_id)?.ok_or_else(|| {
                CoreError::NotFound(format!("Decision with id '{}' not found", id))
            })?;

            if let Some(rep_id_str) = superseded_by {
                let rep_id = Uuid::parse_str(&rep_id_str).map_err(|e| {
                    CoreError::Validation(format!(
                        "Invalid replacing decision UUID '{}': {}",
                        rep_id_str, e
                    ))
                })?;
                decision.supersede(rep_id)?;
            } else if let Some(new_status_str) = status {
                let new_status = DecisionStatus::from_str(&new_status_str)?;
                decision.set_status(new_status)?;
            }

            storage.save_decision(&decision)?;

            let summary = DecisionSummary::from(&decision);
            renderer.success(&summary, || {
                format!(
                    "Updated decision [{}] \"{}\" status to '{}'",
                    summary.id, summary.title, summary.status
                )
            });
            Ok(0)
        }
    }
}
