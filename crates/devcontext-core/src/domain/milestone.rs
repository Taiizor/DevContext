use crate::domain::record::{Record, RecordType};
use crate::error::{CoreError, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Lifecycle status for project milestones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MilestoneStatus {
    Planned,
    Active,
    Completed,
    Cancelled,
}

impl fmt::Display for MilestoneStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Planned => write!(f, "planned"),
            Self::Active => write!(f, "active"),
            Self::Completed => write!(f, "completed"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl std::str::FromStr for MilestoneStatus {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "planned" => Ok(Self::Planned),
            "active" => Ok(Self::Active),
            "completed" => Ok(Self::Completed),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(CoreError::Validation(format!(
                "Invalid milestone status '{}'",
                s
            ))),
        }
    }
}

/// Represents a meaningful checkpoint or goal in the project lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Milestone {
    pub record: Record,
    pub status: MilestoneStatus,
    pub target_date: Option<String>,
    pub completed_date: Option<String>,
    pub description: Option<String>,
}

impl Milestone {
    pub fn new(
        project_id: Uuid,
        title: impl Into<String>,
        status: MilestoneStatus,
        target_date: Option<String>,
        description: Option<String>,
        tags: Vec<String>,
    ) -> Result<Self> {
        let record = Record::new(project_id, RecordType::Milestone, title, tags)?;
        let completed_date = if status == MilestoneStatus::Completed {
            Some(Utc::now().to_rfc3339())
        } else {
            None
        };

        Ok(Self {
            record,
            status,
            target_date,
            completed_date,
            description,
        })
    }

    pub fn set_status(&mut self, status: MilestoneStatus) {
        if status == MilestoneStatus::Completed && self.completed_date.is_none() {
            self.completed_date = Some(Utc::now().to_rfc3339());
        } else if status != MilestoneStatus::Completed {
            self.completed_date = None;
        }
        self.status = status;
        self.record.updated_at = Utc::now().to_rfc3339();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_milestone_completion_date() {
        let proj_id = Uuid::new_v4();
        let mut m = Milestone::new(
            proj_id,
            "v1.0 Release",
            MilestoneStatus::Planned,
            Some("2026-12-01T00:00:00Z".into()),
            Some("Initial public release".into()),
            vec!["release".into()],
        )
        .unwrap();

        assert_eq!(m.completed_date, None);
        m.set_status(MilestoneStatus::Completed);
        assert!(m.completed_date.is_some());
    }
}
