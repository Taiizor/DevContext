use crate::domain::record::{Record, RecordType};
use crate::error::{CoreError, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Lifecycle status for an Architecture Decision Record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionStatus {
    Proposed,
    Accepted,
    Rejected,
    Deprecated,
    Superseded,
}

impl fmt::Display for DecisionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Proposed => write!(f, "proposed"),
            Self::Accepted => write!(f, "accepted"),
            Self::Rejected => write!(f, "rejected"),
            Self::Deprecated => write!(f, "deprecated"),
            Self::Superseded => write!(f, "superseded"),
        }
    }
}

impl std::str::FromStr for DecisionStatus {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "proposed" => Ok(Self::Proposed),
            "accepted" => Ok(Self::Accepted),
            "rejected" => Ok(Self::Rejected),
            "deprecated" => Ok(Self::Deprecated),
            "superseded" => Ok(Self::Superseded),
            _ => Err(CoreError::Validation(format!(
                "Invalid decision status '{}'",
                s
            ))),
        }
    }
}

/// Represents an Architecture Decision Record (ADR).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Decision {
    pub record: Record,
    pub status: DecisionStatus,
    pub context: String,
    pub decision: String,
    pub consequences: String,
    pub alternatives_considered: Option<String>,
    pub superseded_by: Option<Uuid>,
}

impl Decision {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        project_id: Uuid,
        title: impl Into<String>,
        status: DecisionStatus,
        context: impl Into<String>,
        decision: impl Into<String>,
        consequences: impl Into<String>,
        alternatives_considered: Option<String>,
        tags: Vec<String>,
    ) -> Result<Self> {
        let context = context.into();
        let decision_text = decision.into();
        let consequences = consequences.into();

        if context.trim().is_empty() {
            return Err(CoreError::Validation(
                "Decision context cannot be empty".to_string(),
            ));
        }
        if decision_text.trim().is_empty() {
            return Err(CoreError::Validation(
                "Decision statement cannot be empty".to_string(),
            ));
        }
        if consequences.trim().is_empty() {
            return Err(CoreError::Validation(
                "Decision consequences cannot be empty".to_string(),
            ));
        }
        if status == DecisionStatus::Superseded {
            return Err(CoreError::Validation(
                "A new decision cannot be created in superseded state without replacement target"
                    .to_string(),
            ));
        }

        let record = Record::new(project_id, RecordType::Decision, title, tags)?;

        Ok(Self {
            record,
            status,
            context: context.trim().to_string(),
            decision: decision_text.trim().to_string(),
            consequences: consequences.trim().to_string(),
            alternatives_considered: alternatives_considered
                .map(|a| a.trim().to_string())
                .filter(|a| !a.is_empty()),
            superseded_by: None,
        })
    }

    /// Marks this decision as superseded by a newer decision.
    pub fn supersede(&mut self, replacing_decision_id: Uuid) -> Result<()> {
        if replacing_decision_id.is_nil() {
            return Err(CoreError::Validation(
                "Replacing decision id cannot be nil".to_string(),
            ));
        }
        if replacing_decision_id == self.record.id {
            return Err(CoreError::Validation(
                "A decision cannot supersede itself".to_string(),
            ));
        }

        self.status = DecisionStatus::Superseded;
        self.superseded_by = Some(replacing_decision_id);
        self.record.updated_at = Utc::now().to_rfc3339();
        Ok(())
    }

    pub fn set_status(&mut self, status: DecisionStatus) -> Result<()> {
        if status == DecisionStatus::Superseded && self.superseded_by.is_none() {
            return Err(CoreError::Validation(
                "Cannot transition to superseded status without specifying replacing decision id"
                    .to_string(),
            ));
        }
        if status != DecisionStatus::Superseded {
            self.superseded_by = None;
        }
        self.status = status;
        self.record.updated_at = Utc::now().to_rfc3339();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_lifecycle_and_supersede() {
        let proj_id = Uuid::new_v4();
        let mut d1 = Decision::new(
            proj_id,
            "ADR 0001: SQLite storage",
            DecisionStatus::Accepted,
            "We need offline storage",
            "Use SQLite with WAL",
            "Fast, reliable",
            Some("Postgres".into()),
            vec!["architecture".into()],
        )
        .unwrap();

        assert_eq!(d1.status, DecisionStatus::Accepted);
        assert_eq!(d1.superseded_by, None);

        let d2_id = Uuid::new_v4();
        d1.supersede(d2_id).unwrap();

        assert_eq!(d1.status, DecisionStatus::Superseded);
        assert_eq!(d1.superseded_by, Some(d2_id));

        // Self supersede error
        let err = d1.supersede(d1.record.id);
        assert!(matches!(err, Err(CoreError::Validation(_))));
    }
}
