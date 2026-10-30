use super::adapter::GitAdapter;
use crate::error::{CoreError, Result};
use crate::storage::SqliteStorage;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;
use std::str::FromStr;
use uuid::Uuid;

/// Identified category for unified timeline events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimelineEventType {
    Decision,
    Milestone,
    Note,
    LogEntry,
    Commit,
}

impl fmt::Display for TimelineEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Commit => write!(f, "commit"),
            Self::Note => write!(f, "note"),
            Self::Decision => write!(f, "decision"),
            Self::Milestone => write!(f, "milestone"),
            Self::LogEntry => write!(f, "log_entry"),
        }
    }
}

impl FromStr for TimelineEventType {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "commit" => Ok(Self::Commit),
            "note" => Ok(Self::Note),
            "decision" => Ok(Self::Decision),
            "milestone" => Ok(Self::Milestone),
            "log" | "log_entry" => Ok(Self::LogEntry),
            _ => Err(CoreError::Validation(format!(
                "Invalid timeline event type '{}'",
                s
            ))),
        }
    }
}

/// A single unified event on the project timeline.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimelineEvent {
    pub id: String,
    pub event_type: TimelineEventType,
    pub title: String,
    pub timestamp: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Filter criteria for querying the project timeline.
#[derive(Debug, Clone, Default)]
pub struct TimelineFilter {
    pub limit: Option<usize>,
    pub event_type: Option<TimelineEventType>,
    pub since: Option<String>,
    pub until: Option<String>,
}

/// Assembles unified chronological timelines from DevContext records and Git metadata.
pub struct TimelineService;

impl TimelineService {
    pub fn assemble_timeline(
        storage: &SqliteStorage,
        project_id: Uuid,
        project_root: impl AsRef<Path>,
        filter: &TimelineFilter,
    ) -> Result<Vec<TimelineEvent>> {
        let mut events = Vec::new();
        let root = project_root.as_ref();

        // 1. Ingest DevContext Notes
        if filter.event_type.is_none() || filter.event_type == Some(TimelineEventType::Note) {
            let notes = storage.list_notes(project_id, false)?;
            for note in notes {
                events.push(TimelineEvent {
                    id: note.record.id.to_string(),
                    event_type: TimelineEventType::Note,
                    title: note.record.title,
                    timestamp: note.record.created_at,
                    author: None,
                    description: Some(note.content_markdown),
                    tags: note.record.tags,
                    metadata: note.category.map(|c| serde_json::json!({ "category": c })),
                });
            }
        }

        // 2. Ingest DevContext Decisions (ADRs)
        if filter.event_type.is_none() || filter.event_type == Some(TimelineEventType::Decision) {
            let decisions = storage.list_decisions(project_id)?;
            for dec in decisions {
                events.push(TimelineEvent {
                    id: dec.record.id.to_string(),
                    event_type: TimelineEventType::Decision,
                    title: format!(
                        "{} [{}]",
                        dec.record.title,
                        dec.status.to_string().to_uppercase()
                    ),
                    timestamp: dec.record.created_at,
                    author: None,
                    description: Some(format!(
                        "Context: {}\nDecision: {}",
                        dec.context, dec.decision
                    )),
                    tags: dec.record.tags,
                    metadata: Some(serde_json::json!({
                        "status": dec.status.to_string(),
                        "superseded_by": dec.superseded_by.map(|u| u.to_string())
                    })),
                });
            }
        }

        // 3. Ingest DevContext Milestones
        if filter.event_type.is_none() || filter.event_type == Some(TimelineEventType::Milestone) {
            let milestones = storage.list_milestones(project_id)?;
            for m in milestones {
                events.push(TimelineEvent {
                    id: m.record.id.to_string(),
                    event_type: TimelineEventType::Milestone,
                    title: format!(
                        "{} [{}]",
                        m.record.title,
                        m.status.to_string().to_uppercase()
                    ),
                    timestamp: m.record.created_at,
                    author: None,
                    description: m.description,
                    tags: m.record.tags,
                    metadata: Some(serde_json::json!({
                        "status": m.status.to_string(),
                        "target_date": m.target_date,
                        "completed_date": m.completed_date
                    })),
                });
            }
        }

        // 4. Ingest DevContext Development Log Entries
        if filter.event_type.is_none() || filter.event_type == Some(TimelineEventType::LogEntry) {
            let logs = storage.list_log_entries(project_id)?;
            for log in logs {
                events.push(TimelineEvent {
                    id: log.record.id.to_string(),
                    event_type: TimelineEventType::LogEntry,
                    title: log.record.title,
                    timestamp: log.event_date,
                    author: None,
                    description: Some(log.summary),
                    tags: log.record.tags,
                    metadata: log.details.map(|d| serde_json::json!({ "details": d })),
                });
            }
        }

        // 5. Ingest Git Commits (if Git is present and commits match filter)
        if (filter.event_type.is_none() || filter.event_type == Some(TimelineEventType::Commit))
            && GitAdapter::is_git_repository(root)
        {
            // Request bounded commits from Git
            let commits = GitAdapter::get_commits(
                root,
                filter.limit.map(|l| l * 2), // small buffer for interleaving
                filter.since.as_deref(),
                filter.until.as_deref(),
            )?;

            for commit in commits {
                events.push(TimelineEvent {
                    id: commit.hash.clone(),
                    event_type: TimelineEventType::Commit,
                    title: commit.message,
                    timestamp: commit.timestamp,
                    author: Some(format!("{} <{}>", commit.author_name, commit.author_email)),
                    description: None,
                    tags: vec![],
                    metadata: Some(serde_json::json!({
                        "short_hash": commit.short_hash,
                        "hash": commit.hash
                    })),
                });
            }
        }

        // 6. Filter by Date Range
        if let Some(since) = &filter.since {
            events.retain(|e| &e.timestamp >= since);
        }
        if let Some(until) = &filter.until {
            events.retain(|e| &e.timestamp <= until);
        }

        // 7. Stable Chronological Sorting (Newest First)
        // Primary: timestamp descending
        // Secondary: event_type priority
        // Tertiary: id lexicographical
        events.sort_by(|a, b| {
            b.timestamp
                .cmp(&a.timestamp)
                .then_with(|| a.event_type.cmp(&b.event_type))
                .then_with(|| a.id.cmp(&b.id))
        });

        // 8. Apply limit
        if let Some(limit) = filter.limit {
            if events.len() > limit {
                events.truncate(limit);
            }
        }

        Ok(events)
    }
}
