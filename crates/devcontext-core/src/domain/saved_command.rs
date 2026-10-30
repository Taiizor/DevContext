use crate::domain::record::{Record, RecordType};
use crate::error::{CoreError, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a developer-saved reusable command snippet.
///
/// NOTE: Commands are stored strictly as text and are NEVER executed automatically.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SavedCommand {
    pub record: Record,
    pub command_text: String,
    pub explanation: Option<String>,
    pub working_directory: Option<String>,
}

impl SavedCommand {
    pub fn new(
        project_id: Uuid,
        title: impl Into<String>,
        command_text: impl Into<String>,
        explanation: Option<String>,
        working_directory: Option<String>,
        tags: Vec<String>,
    ) -> Result<Self> {
        let cmd = command_text.into();
        let trimmed_cmd = cmd.trim();
        if trimmed_cmd.is_empty() {
            return Err(CoreError::Validation(
                "Command text cannot be empty".to_string(),
            ));
        }

        let record = Record::new(project_id, RecordType::SavedCommand, title, tags)?;

        Ok(Self {
            record,
            command_text: trimmed_cmd.to_string(),
            explanation: explanation
                .map(|e| e.trim().to_string())
                .filter(|e| !e.is_empty()),
            working_directory: working_directory
                .map(|w| w.trim().to_string())
                .filter(|w| !w.is_empty()),
        })
    }

    /// Basic heuristic to detect potential credentials or secrets inside command text.
    pub fn contains_potential_secret(&self) -> bool {
        let lower = self.command_text.to_lowercase();
        lower.contains("bearer ")
            || lower.contains("authorization:")
            || lower.contains("ghp_")
            || lower.contains("glpat-")
            || lower.contains("aws_secret_access_key")
            || lower.contains("password=")
            || lower.contains("api_key=")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_saved_command_validation_and_secret_detection() {
        let proj_id = Uuid::new_v4();
        let err = SavedCommand::new(proj_id, "Run tests", "", None, None, vec![]);
        assert!(matches!(err, Err(CoreError::Validation(_))));

        let normal = SavedCommand::new(
            proj_id,
            "Run integration tests",
            "cargo test --test integration",
            Some("Runs all integration fixtures".into()),
            None,
            vec!["test".into()],
        )
        .unwrap();
        assert!(!normal.contains_potential_secret());

        let risky = SavedCommand::new(
            proj_id,
            "Deploy with token",
            "curl -H 'Authorization: Bearer secret-token' https://api.example.com",
            None,
            None,
            vec!["deploy".into()],
        )
        .unwrap();
        assert!(risky.contains_potential_secret());
    }
}
