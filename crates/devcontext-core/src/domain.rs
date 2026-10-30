use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Supported record types in DevContext.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordType {
    Note,
    Decision,
    Milestone,
    LogEntry,
    SavedCommand,
}

/// Project identity and root metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub created_at: String,
}

impl Project {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_creation() {
        let project = Project::new("test-project");
        assert_eq!(project.name, "test-project");
        assert!(!project.id.is_nil());
        assert!(!project.created_at.is_empty());
    }
}
