use crate::args::TimelineArgs;
use crate::output::OutputRenderer;
use devcontext_core::error::{CoreError, Result};
use devcontext_core::git::{TimelineEventType, TimelineFilter, TimelineService};
use devcontext_core::workspace::WorkspaceManager;
use std::path::PathBuf;
use std::str::FromStr;

pub fn handle_timeline(
    project_dir: Option<PathBuf>,
    args: TimelineArgs,
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

    let parsed_type = match args.event_type {
        Some(t) => Some(TimelineEventType::from_str(&t)?),
        None => None,
    };

    let filter = TimelineFilter {
        limit: Some(args.limit),
        event_type: parsed_type,
        since: args.since,
        until: args.until,
    };

    let events = TimelineService::assemble_timeline(&storage, project.id, &root, &filter)?;

    renderer.success(&events, || {
        if events.is_empty() {
            return "No timeline events match the criteria.".to_string();
        }

        let mut out = format!("Project Timeline ({} events):\n\n", events.len());
        for ev in &events {
            let type_label = ev.event_type.to_string().to_uppercase();
            out.push_str(&format!(
                "[{}] {}: {}\n",
                ev.timestamp, type_label, ev.title
            ));

            if let Some(author) = &ev.author {
                out.push_str(&format!("  Author: {}\n", author));
            }

            if let Some(desc) = &ev.description {
                let first_line = desc.lines().next().unwrap_or("").trim();
                if !first_line.is_empty() && first_line != ev.title {
                    out.push_str(&format!("  {}\n", first_line));
                }
            }

            out.push('\n');
        }

        out.trim_end().to_string()
    });

    Ok(0)
}
