use crate::error::{CoreError, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Typed category of reference linking from a DevContext record to a target entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceType {
    File,
    Folder,
    Symbol,
    GitCommit,
    GitBranch,
    Url,
    Command,
    Record,
}

impl fmt::Display for ReferenceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::File => write!(f, "file"),
            Self::Folder => write!(f, "folder"),
            Self::Symbol => write!(f, "symbol"),
            Self::GitCommit => write!(f, "git_commit"),
            Self::GitBranch => write!(f, "git_branch"),
            Self::Url => write!(f, "url"),
            Self::Command => write!(f, "command"),
            Self::Record => write!(f, "record"),
        }
    }
}

impl std::str::FromStr for ReferenceType {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "file" => Ok(Self::File),
            "folder" => Ok(Self::Folder),
            "symbol" => Ok(Self::Symbol),
            "git_commit" => Ok(Self::GitCommit),
            "git_branch" => Ok(Self::GitBranch),
            "url" => Ok(Self::Url),
            "command" => Ok(Self::Command),
            "record" => Ok(Self::Record),
            _ => Err(CoreError::Validation(format!(
                "Invalid reference type '{}'",
                s
            ))),
        }
    }
}

/// Represents a typed reference link attached to any DevContext record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Reference {
    pub id: Uuid,
    pub record_id: Uuid,
    pub ref_type: ReferenceType,
    pub target: String,
    pub label: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: String,
}

impl Reference {
    pub fn new(
        record_id: Uuid,
        ref_type: ReferenceType,
        target: impl Into<String>,
        label: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> Result<Self> {
        let target = target.into();
        let trimmed_target = target.trim();
        if trimmed_target.is_empty() {
            return Err(CoreError::Validation(
                "Reference target cannot be empty".to_string(),
            ));
        }
        if record_id.is_nil() {
            return Err(CoreError::Validation(
                "Reference record_id cannot be nil".to_string(),
            ));
        }

        Ok(Self {
            id: Uuid::new_v4(),
            record_id,
            ref_type,
            target: trimmed_target.to_string(),
            label: label
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty()),
            metadata,
            created_at: Utc::now().to_rfc3339(),
        })
    }

    pub fn with_id(
        id: Uuid,
        record_id: Uuid,
        ref_type: ReferenceType,
        target: impl Into<String>,
        label: Option<String>,
        metadata: Option<serde_json::Value>,
        created_at: impl Into<String>,
    ) -> Result<Self> {
        let target = target.into();
        let trimmed_target = target.trim();
        if trimmed_target.is_empty() {
            return Err(CoreError::Validation(
                "Reference target cannot be empty".to_string(),
            ));
        }
        if id.is_nil() || record_id.is_nil() {
            return Err(CoreError::Validation(
                "Reference id or record_id cannot be nil".to_string(),
            ));
        }

        Ok(Self {
            id,
            record_id,
            ref_type,
            target: trimmed_target.to_string(),
            label: label
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty()),
            metadata,
            created_at: created_at.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reference_validation() {
        let rec_id = Uuid::new_v4();
        let err = Reference::new(rec_id, ReferenceType::File, "   ", None, None);
        assert!(matches!(err, Err(CoreError::Validation(_))));

        let valid = Reference::new(
            rec_id,
            ReferenceType::File,
            "src/storage/mod.rs",
            Some("Storage module".into()),
            None,
        )
        .unwrap();

        assert_eq!(valid.ref_type, ReferenceType::File);
        assert_eq!(valid.target, "src/storage/mod.rs");
        assert_eq!(valid.label, Some("Storage module".into()));
    }
}
