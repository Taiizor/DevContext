use crate::args::ImportArgs;
use crate::output::OutputRenderer;
use devcontext_core::error::{CoreError, Result};
use devcontext_core::export::{ConflictStrategy, ExportService, ImportOptions};
use devcontext_core::workspace::WorkspaceManager;
use std::path::PathBuf;
use std::str::FromStr;

pub fn handle_import(
    project_dir: Option<PathBuf>,
    args: ImportArgs,
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

    let on_conflict = ConflictStrategy::from_str(&args.on_conflict)?;
    let options = ImportOptions {
        on_conflict,
        create_backup: !args.no_backup,
        dry_run: args.dry_run,
    };

    let result = ExportService::import_bundle(&root, &args.bundle_path, &options)?;

    renderer.success(&result, || {
        if result.dry_run {
            format!(
                "=== DevContext Import Dry-Run / Preview ===\n  Target Bundle:    '{}'\n  Total Incoming:   {}\n  Would Import:     {}\n  Would Skip:       {}\n  Would Overwrite:  {}\n  Would Merge:      {}\n  Conflicting IDs:  {}\n  Conflict Policy:  {}\n\nNo changes made to project database (dry-run mode).",
                args.bundle_path.display(),
                result.total_incoming,
                result.imported_count,
                result.skipped_count,
                result.overwritten_count,
                result.merged_count,
                result.conflicts_count,
                on_conflict
            )
        } else {
            let mut msg = format!(
                "Import completed: {} imported, {} overwritten, {} merged, {} skipped (total incoming: {})",
                result.imported_count, result.overwritten_count, result.merged_count, result.skipped_count, result.total_incoming
            );
            if let Some(backup) = &result.backup_path {
                msg.push_str(&format!("\nAutomatic pre-import backup snapshot saved to: '{}'", backup));
            }
            msg
        }
    });

    Ok(0)
}
