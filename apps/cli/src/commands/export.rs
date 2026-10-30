use crate::args::ExportArgs;
use crate::output::OutputRenderer;
use devcontext_core::error::{CoreError, Result};
use devcontext_core::export::{ExportFormat, ExportOptions, ExportService};
use devcontext_core::workspace::WorkspaceManager;
use serde::Serialize;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Serialize)]
pub struct ExportOutput {
    pub project_name: String,
    pub format: String,
    pub output_path: String,
    pub total_records: usize,
    pub schema_version: String,
}

pub fn handle_export(
    project_dir: Option<PathBuf>,
    args: ExportArgs,
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

    if args.preview {
        let preview = ExportService::preview_export(&root, args.redact_paths)?;
        renderer.success(&preview, || {
            format!(
                "=== DevContext Export Preview for '{}' ===\n  Notes:            {}\n  Decisions (ADRs): {}\n  Milestones:       {}\n  Logs:             {}\n  Saved Commands:   {}\n  Total Records:    {}\n  References:       {}\n  Path Redaction:   {}",
                preview.project_name,
                preview.notes_count,
                preview.decisions_count,
                preview.milestones_count,
                preview.logs_count,
                preview.commands_count,
                preview.total_records,
                preview.references_count,
                if preview.redact_paths { "Enabled" } else { "Disabled" }
            )
        });
        return Ok(0);
    }

    let format = ExportFormat::from_str(&args.format)?;
    let options = ExportOptions {
        format,
        output_path: args.output,
        redact_paths: args.redact_paths,
    };

    let (bundle, path) = ExportService::export_project(&root, &options)?;

    let output = ExportOutput {
        project_name: bundle.project.name,
        format: format.to_string(),
        output_path: path.display().to_string(),
        total_records: bundle.records.len(),
        schema_version: bundle.schema_version,
    };

    renderer.success(&output, || {
        format!(
            "Successfully exported {} records to '{}' (format: {})",
            output.total_records, output.output_path, output.format
        )
    });

    Ok(0)
}
