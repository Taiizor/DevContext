use crate::args::DeleteDataArgs;
use crate::output::OutputRenderer;
use devcontext_core::error::{CoreError, Result};
use devcontext_core::export::ExportService;
use devcontext_core::workspace::WorkspaceManager;
use std::path::PathBuf;

pub fn handle_delete_data(
    project_dir: Option<PathBuf>,
    args: DeleteDataArgs,
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

    if !args.force {
        return Err(CoreError::Validation(
            "Destructive project data deletion requires explicit '--force' flag to confirm."
                .to_string(),
        ));
    }

    let result = ExportService::delete_project_data(&root, true, !args.no_backup)?;

    renderer.success(&result, || {
        let mut msg = format!(
            "All project memory cleared for '{}' ({} records deleted).",
            result.project_name, result.deleted_records_count
        );
        if let Some(backup) = &result.backup_path {
            msg.push_str(&format!(
                "\nAutomatic pre-deletion backup snapshot saved to: '{}'",
                backup
            ));
        }
        msg
    });

    Ok(0)
}
