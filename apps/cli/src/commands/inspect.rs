use crate::args::InspectArgs;
use crate::output::OutputRenderer;
use devcontext_core::error::{CoreError, Result};
use devcontext_core::export::ExportService;
use devcontext_core::workspace::WorkspaceManager;
use std::path::PathBuf;

pub fn handle_inspect(
    project_dir: Option<PathBuf>,
    args: InspectArgs,
    renderer: &OutputRenderer,
) -> Result<i32> {
    let start_dir = match project_dir {
        Some(p) => p,
        None => std::env::current_dir()?,
    };

    let root = WorkspaceManager::find_project_root(&start_dir).ok_or_else(|| {
        CoreError::NotFound(format!(
            "No DevContext project found in '{}' or parent directories. Run 'devcontext init' first.",
            start_dir.display()
        ))
    })?;

    let report = ExportService::inspect_source(&root, args.file.as_deref())?;

    renderer.success(&report, || {
        let mut msg = format!(
            "=== DevContext Data Inspection Report ===\n  Source Type:      {}\n  Project Name:     {}\n  Project ID:       {}\n",
            report.source_type, report.project_name, report.project_id
        );
        if let Some(v) = &report.schema_version {
            msg.push_str(&format!("  Schema Version:   {}\n", v));
        }
        msg.push_str(&format!(
            "  Notes:            {}\n  Decisions (ADRs): {}\n  Milestones:       {}\n  Logs:             {}\n  Saved Commands:   {}\n  Total Records:    {}\n  References:       {}",
            report.notes_count,
            report.decisions_count,
            report.milestones_count,
            report.logs_count,
            report.commands_count,
            report.total_records,
            report.references_count
        ));
        if report.source_type != "database" {
            msg.push_str(&format!("\n  Conflicting IDs:  {}", report.conflicts_with_existing));
        }
        msg
    });

    Ok(0)
}
