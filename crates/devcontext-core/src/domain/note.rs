use crate::domain::record::{Record, RecordType};
use crate::error::{CoreError, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a developer-authored context note.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Note {
    pub record: Record,
    pub content_markdown: String,
    pub category: Option<String>,
    pub is_pinned: bool,
    pub is_archived: bool,
}

impl Note {
    pub fn new(
        project_id: Uuid,
        title: impl Into<String>,
        content_markdown: impl Into<String>,
        tags: Vec<String>,
        category: Option<String>,
    ) -> Result<Self> {
        let content = content_markdown.into();
        let trimmed_content = content.trim();
        if trimmed_content.is_empty() {
            return Err(CoreError::Validation(
                "Note content cannot be empty".to_string(),
            ));
        }

        let record = Record::new(project_id, RecordType::Note, title, tags)?;
        let category = category
            .map(|c| c.trim().to_string())
            .filter(|c| !c.is_empty());

        Ok(Self {
            record,
            content_markdown: trimmed_content.to_string(),
            category,
            is_pinned: false,
            is_archived: false,
        })
    }

    pub fn update_content(&mut self, content: impl Into<String>) -> Result<()> {
        let content = content.into();
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return Err(CoreError::Validation(
                "Updated note content cannot be empty".to_string(),
            ));
        }
        self.content_markdown = trimmed.to_string();
        self.record.updated_at = Utc::now().to_rfc3339();
        Ok(())
    }

    pub fn set_pinned(&mut self, is_pinned: bool) {
        self.is_pinned = is_pinned;
        self.record.updated_at = Utc::now().to_rfc3339();
    }

    pub fn set_archived(&mut self, is_archived: bool) {
        self.is_archived = is_archived;
        self.record.updated_at = Utc::now().to_rfc3339();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_validation() {
        let proj_id = Uuid::new_v4();
        let err = Note::new(proj_id, "Title", "  ", vec![], None);
        assert!(matches!(err, Err(CoreError::Validation(_))));

        let mut note = Note::new(
            proj_id,
            "Valid Note",
            "Initial content",
            vec!["doc".into()],
            Some("backend".into()),
        )
        .unwrap();

        assert_eq!(note.record.title, "Valid Note");
        assert_eq!(note.content_markdown, "Initial content");
        assert_eq!(note.category, Some("backend".into()));

        note.update_content("Updated content").unwrap();
        assert_eq!(note.content_markdown, "Updated content");
    }
}
