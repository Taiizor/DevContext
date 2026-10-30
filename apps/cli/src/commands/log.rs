use crate::args::LogCommands;
use crate::output::OutputRenderer;
use devcontext_core::domain::LogEntry;
use devcontext_core::error::{CoreError, Result};
use devcontext_core::workspace::WorkspaceManager;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
pub struct LogSummary {
    pub id: String,
    pub title: String,
    pub event_date: String,
    pub summary: String,
}

impl From<&LogEntry> for LogSummary {
    fn from(l: &LogEntry) -> Self {
        Self {
            id: l.record.id.to_string(),
            title: l.record.title.clone(),
            event_date: l.event_date.clone(),
            summary: l.summary.clone(),
        }
    }
}

pub fn handle_log(
    project_dir: Option<PathBuf>,
    cmd: LogCommands,
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
        LogCommands::Add {
            title,
            summary,
            details,
            date,
            tag,
        } => {
            let entry = LogEntry::new(project.id, title, summary, details, date, tag)?;
            storage.save_log_entry(&entry)?;

            let sum = LogSummary::from(&entry);
            renderer.success(&sum, || {
                format!(
                    "Recorded log entry [{}] \"{}\" on {}",
                    sum.id, sum.title, sum.event_date
                )
            });
            Ok(0)
        }

        LogCommands::List => {
            let logs = storage.list_log_entries(project.id)?;
            let summaries: Vec<LogSummary> = logs.iter().map(LogSummary::from).collect();

            renderer.success(&summaries, || {
                if summaries.is_empty() {
                    return "No log entries recorded.".to_string();
                }
                let mut out = format!("Development Log ({}):\n", summaries.len());
                for l in &summaries {
                    out.push_str(&format!(
                        "- [{}] {} \"{}\" — {}\n",
                        l.event_date, l.id, l.title, l.summary
                    ));
                }
                out
            });
            Ok(0)
        }
    }
}
