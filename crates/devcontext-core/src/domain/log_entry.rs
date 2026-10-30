use crate::domain::record::{Record, RecordType};
use crate::error::{CoreError, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents an ad-hoc development event (e.g. migration, debugging breakthrough, release).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogEntry {
    pub record: Record,
    pub event_date: String,
    pub summary: String,
    pub details: Option<String>,
}

impl LogEntry {
    pub fn new(
        project_id: Uuid,
        title: impl Into<String>,
        summary: impl Into<String>,
        details: Option<String>,
        event_date: Option<String>,
        tags: Vec<String>,
    ) -> Result<Self> {
        let summary = summary.into();
        let trimmed_summary = summary.trim();
        if trimmed_summary.is_empty() {
            return Err(CoreError::Validation(
                "Log entry summary cannot be empty".to_string(),
            ));
        }

        let record = Record::new(project_id, RecordType::LogEntry, title, tags)?;
        let event_date = event_date.unwrap_or_else(|| Utc::now().to_rfc3339());

        Ok(Self {
            record,
            event_date,
            summary: trimmed_summary.to_string(),
            details: details
                .map(|d| d.trim().to_string())
                .filter(|d| !d.is_empty()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_entry_validation() {
        let proj_id = Uuid::new_v4();
        let res = LogEntry::new(proj_id, "Title", "  ", None, None, vec![]);
        assert!(matches!(res, Err(CoreError::Validation(_))));

        let valid = LogEntry::new(
            proj_id,
            "SQLite Migration Complete",
            "Successfully migrated from v0 to v1",
            Some("Detailed notes here".into()),
            None,
            vec!["migration".into()],
        )
        .unwrap();

        assert_eq!(valid.summary, "Successfully migrated from v0 to v1");
        assert!(!valid.event_date.is_empty());
    }
}
