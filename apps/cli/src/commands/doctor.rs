use crate::output::OutputRenderer;
use devcontext_core::error::Result;
use devcontext_core::storage::{BackupService, DbConnection, MigrationRunner};
use devcontext_core::workspace::WorkspaceManager;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
pub struct DoctorOutput {
    pub cli_version: String,
    pub protocol_version: String,
    pub project_detected: bool,
    pub project_info: Option<ProjectDoctorInfo>,
}

#[derive(Serialize)]
pub struct ProjectDoctorInfo {
    pub root_path: String,
    pub schema_version: i64,
    pub journal_mode: String,
    pub foreign_keys_valid: bool,
    pub integrity_status: String,
}

pub fn handle_doctor(project_dir: Option<PathBuf>, renderer: &OutputRenderer) -> Result<i32> {
    let start_dir = match project_dir {
        Some(p) => p,
        None => std::env::current_dir()?,
    };

    let maybe_root = WorkspaceManager::find_project_root(&start_dir);

    let mut project_info = None;
    if let Some(root) = maybe_root {
        if let Ok((_, storage)) = WorkspaceManager::open_project(&root) {
            let conn = DbConnection::open(root.join(".devcontext").join("devcontext.db"))?;
            let schema_version = MigrationRunner::current_version(&conn).unwrap_or(0);
            let journal_mode =
                DbConnection::get_journal_mode(&conn).unwrap_or_else(|_| "unknown".into());
            let foreign_keys_valid = BackupService::verify_foreign_keys(&storage).is_ok();
            let integrity_status = match BackupService::verify_integrity(&storage) {
                Ok(v) => v.join(", "),
                Err(e) => format!("Error: {}", e),
            };

            project_info = Some(ProjectDoctorInfo {
                root_path: root.display().to_string(),
                schema_version,
                journal_mode,
                foreign_keys_valid,
                integrity_status,
            });
        }
    }

    let output = DoctorOutput {
        cli_version: devcontext_core::version().to_string(),
        protocol_version: "1.0".to_string(),
        project_detected: project_info.is_some(),
        project_info,
    };

    renderer.success(&output, || {
        let mut msg = format!(
            "DevContext Diagnostic Doctor\nCLI Version: {}\nProtocol Version: {}\n",
            output.cli_version, output.protocol_version
        );

        if let Some(info) = &output.project_info {
            msg.push_str(&format!(
                "\nProject Root: {}\nSchema Version: {}\nJournal Mode: {}\nForeign Keys: {}\nIntegrity Status: {}",
                info.root_path,
                info.schema_version,
                info.journal_mode,
                if info.foreign_keys_valid { "Valid (enforced)" } else { "FAILED" },
                info.integrity_status
            ));
        } else {
            msg.push_str("\nNo DevContext project detected in current directory context.");
        }

        msg
    });

    Ok(0)
}
