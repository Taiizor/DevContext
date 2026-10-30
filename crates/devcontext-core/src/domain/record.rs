use crate::error::{CoreError, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Supported record types in DevContext.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordType {
    Note,
    Decision,
    Milestone,
    LogEntry,
    SavedCommand,
}

impl fmt::Display for RecordType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Note => write!(f, "note"),
            Self::Decision => write!(f, "decision"),
            Self::Milestone => write!(f, "milestone"),
            Self::LogEntry => write!(f, "log_entry"),
            Self::SavedCommand => write!(f, "saved_command"),
        }
    }
}

impl std::str::FromStr for RecordType {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "note" => Ok(Self::Note),
            "decision" => Ok(Self::Decision),
            "milestone" => Ok(Self::Milestone),
            "log_entry" => Ok(Self::LogEntry),
            "saved_command" => Ok(Self::SavedCommand),
            _ => Err(CoreError::Validation(format!(
                "Invalid record type '{}'",
                s
            ))),
        }
    }
}

/// The common metadata envelope shared across all DevContext records.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Record {
    pub id: Uuid,
    pub project_id: Uuid,
    pub record_type: RecordType,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub tags: Vec<String>,
    pub is_deleted: bool,
}

impl Record {
    pub fn new(
        project_id: Uuid,
        record_type: RecordType,
        title: impl Into<String>,
        tags: Vec<String>,
    ) -> Result<Self> {
        let title = title.into();
        let trimmed_title = title.trim();
        if trimmed_title.is_empty() {
            return Err(CoreError::Validation(
                "Record title cannot be empty".to_string(),
            ));
        }
        if project_id.is_nil() {
            return Err(CoreError::Validation(
                "Record project_id cannot be nil".to_string(),
            ));
        }

        let now = Utc::now().to_rfc3339();
        let normalized_tags = Self::normalize_tags(tags);

        Ok(Self {
            id: Uuid::new_v4(),
            project_id,
            record_type,
            title: trimmed_title.to_string(),
            created_at: now.clone(),
            updated_at: now,
            tags: normalized_tags,
            is_deleted: false,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_id(
        id: Uuid,
        project_id: Uuid,
        record_type: RecordType,
        title: impl Into<String>,
        created_at: impl Into<String>,
        updated_at: impl Into<String>,
        tags: Vec<String>,
        is_deleted: bool,
    ) -> Result<Self> {
        let title = title.into();
        let trimmed_title = title.trim();
        if trimmed_title.is_empty() {
            return Err(CoreError::Validation(
                "Record title cannot be empty".to_string(),
            ));
        }
        if id.is_nil() {
            return Err(CoreError::Validation("Record id cannot be nil".to_string()));
        }
        if project_id.is_nil() {
            return Err(CoreError::Validation(
                "Record project_id cannot be nil".to_string(),
            ));
        }

        Ok(Self {
            id,
            project_id,
            record_type,
            title: trimmed_title.to_string(),
            created_at: created_at.into(),
            updated_at: updated_at.into(),
            tags: Self::normalize_tags(tags),
            is_deleted,
        })
    }

    pub fn update_title(&mut self, new_title: impl Into<String>) -> Result<()> {
        let new_title = new_title.into();
        let trimmed = new_title.trim();
        if trimmed.is_empty() {
            return Err(CoreError::Validation(
                "Updated title cannot be empty".to_string(),
            ));
        }
        self.title = trimmed.to_string();
        self.updated_at = Utc::now().to_rfc3339();
        Ok(())
    }

    pub fn set_tags(&mut self, tags: Vec<String>) {
        self.tags = Self::normalize_tags(tags);
        self.updated_at = Utc::now().to_rfc3339();
    }

    fn normalize_tags(tags: Vec<String>) -> Vec<String> {
        let mut result = Vec::new();
        for tag in tags {
            let trimmed = tag.trim().to_lowercase();
            if !trimmed.is_empty() && !result.contains(&trimmed) {
                result.push(trimmed);
            }
        }
        result.sort();
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let proj_id = Uuid::new_v4();
        let res = Record::new(proj_id, RecordType::Note, "", vec![]);
        assert!(matches!(res, Err(CoreError::Validation(_))));

        let res = Record::new(Uuid::nil(), RecordType::Note, "Valid Title", vec![]);
        assert!(matches!(res, Err(CoreError::Validation(_))));

        let valid = Record::new(
            proj_id,
            RecordType::Note,
            " My Title ",
            vec![
                "Architecture".into(),
                "  storage  ".into(),
                "architecture".into(),
            ],
        )
        .unwrap();

        assert_eq!(valid.title, "My Title");
        assert_eq!(valid.tags, vec!["architecture", "storage"]);
    }
}
