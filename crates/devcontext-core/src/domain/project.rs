use crate::error::{CoreError, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a distinct project workspace recognized by DevContext.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub created_at: String,
    pub vcs_identifier: Option<String>,
}

impl Project {
    pub fn new(name: impl Into<String>, vcs_identifier: Option<String>) -> Result<Self> {
        let name = name.into();
        let trimmed_name = name.trim();
        if trimmed_name.is_empty() {
            return Err(CoreError::Validation(
                "Project name cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            id: Uuid::new_v4(),
            name: trimmed_name.to_string(),
            created_at: Utc::now().to_rfc3339(),
            vcs_identifier,
        })
    }

    pub fn with_id(
        id: Uuid,
        name: impl Into<String>,
        created_at: impl Into<String>,
        vcs_identifier: Option<String>,
    ) -> Result<Self> {
        let name = name.into();
        let trimmed_name = name.trim();
        if trimmed_name.is_empty() {
            return Err(CoreError::Validation(
                "Project name cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            id,
            name: trimmed_name.to_string(),
            created_at: created_at.into(),
            vcs_identifier,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_project_creation() {
        let proj = Project::new("DevContext", Some("git-hash-123".into())).unwrap();
        assert_eq!(proj.name, "DevContext");
        assert_eq!(proj.vcs_identifier, Some("git-hash-123".into()));
        assert!(!proj.id.is_nil());
        assert!(!proj.created_at.is_empty());
    }

    #[test]
    fn test_empty_project_name_fails() {
        let res = Project::new("   ", None);
        assert!(matches!(res, Err(CoreError::Validation(_))));
    }
}
