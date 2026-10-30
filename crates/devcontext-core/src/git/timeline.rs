use super::adapter::GitAdapter;
use crate::error::{CoreError, Result};
use crate::storage::SqliteStorage;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;
use std::str::FromStr;
use uuid::Uuid;

/// Upper limit of Git commits queried when timeline limit is unspecified.
pub const DEFAULT_MAX_COMMITS_UNBOUNDED: usize = 1000;

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
            // Bounded commit retrieval:
            // - If user specifies limit L, fetch up to 2*L commits to allow interleaving with DevContext records
            // - If limit is omitted, cap at DEFAULT_MAX_COMMITS_UNBOUNDED to prevent unbounded memory loading on 100k+ commit repositories
            let commit_limit = filter
                .limit
                .map(|l| l.saturating_mul(2))
                .or(Some(DEFAULT_MAX_COMMITS_UNBOUNDED));

            let commits = GitAdapter::get_commits(
                root,
                commit_limit,
                filter.since.as_deref(),
                filter.until.as_deref(),
            )?;

            for commit in commits {
                let author = match (
                    commit.author_name.is_empty(),
                    commit.author_email.is_empty(),
                ) {
                    (false, false) => {
                        Some(format!("{} <{}>", commit.author_name, commit.author_email))
                    }
                    (false, true) => Some(commit.author_name.clone()),
                    (true, false) => Some(commit.author_email.clone()),
                    (true, true) => None,
                };

                // Split message into first line (title) and remainder (body/description)
                let mut lines = commit.message.lines();
                let first_line = lines.next().unwrap_or("").trim().to_string();
                let title = if first_line.is_empty() {
                    commit.short_hash.clone()
                } else {
                    first_line
                };
                let rest: Vec<&str> = lines.collect();
                let body = rest.join("\n").trim().to_string();
                let description = if body.is_empty() { None } else { Some(body) };

                events.push(TimelineEvent {
                    id: commit.hash.clone(),
                    event_type: TimelineEventType::Commit,
                    title,
                    timestamp: commit.timestamp,
                    author,
                    description,
                    tags: vec![],
                    metadata: Some(serde_json::json!({
                        "short_hash": commit.short_hash,
                        "hash": commit.hash
                    })),
                });
            }
        }

        // 6. Predictable Date Range Filtering
        let since_bound = filter
            .since
            .as_deref()
            .and_then(|s| parse_bound_date(s, false));
        let until_bound = filter
            .until
            .as_deref()
            .and_then(|u| parse_bound_date(u, true));

        events.retain(|e| {
            if let (Some(bound), Some(ev_dt)) = (&since_bound, parse_event_timestamp(&e.timestamp))
            {
                if &ev_dt < bound {
                    return false;
                }
            } else if let Some(since) = &filter.since {
                if &e.timestamp < since {
                    return false;
                }
            }

            if let (Some(bound), Some(ev_dt)) = (&until_bound, parse_event_timestamp(&e.timestamp))
            {
                if &ev_dt > bound {
                    return false;
                }
            } else if let Some(until) = &filter.until {
                if &e.timestamp > until {
                    return false;
                }
            }

            true
        });

        // 7. Stable Chronological Sorting (Newest First)
        // Primary: timestamp descending
        // Secondary: event_type priority
        // Tertiary: id lexicographical tie-breaker (stable between runs)
        // Note: We use slice::sort_by with stable sort or index to preserve topological order within same type
        let mut indexed_events: Vec<(TimelineEvent, usize)> = events
            .into_iter()
            .enumerate()
            .map(|(idx, ev)| (ev, idx))
            .collect();

        indexed_events.sort_by(|(a, a_idx), (b, b_idx)| {
            b.timestamp
                .cmp(&a.timestamp)
                .then_with(|| a.event_type.cmp(&b.event_type))
                .then_with(|| a_idx.cmp(b_idx))
        });

        let mut events: Vec<TimelineEvent> = indexed_events.into_iter().map(|(ev, _)| ev).collect();

        // 8. Apply limit
        if let Some(limit) = filter.limit {
            if events.len() > limit {
                events.truncate(limit);
            }
        }

        Ok(events)
    }
}

/// Parses an input boundary filter string. If given a date `YYYY-MM-DD`,
/// when `is_end_of_day` is true, sets time to `23:59:59.999999999Z` (inclusive of day).
fn parse_bound_date(s: &str, is_end_of_day: bool) -> Option<DateTime<Utc>> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }
    if let Ok(ndt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        return Some(ndt.and_utc());
    }
    if let Ok(nd) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        if is_end_of_day {
            return Some(nd.and_hms_nano_opt(23, 59, 59, 999_999_999)?.and_utc());
        } else {
            return Some(nd.and_hms_opt(0, 0, 0)?.and_utc());
        }
    }
    None
}

/// Parses an event timestamp string into a UTC DateTime for accurate comparison.
fn parse_event_timestamp(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .ok()
        .or_else(|| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
                .map(|ndt| ndt.and_utc())
                .ok()
        })
        .or_else(|| {
            chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc())
                .ok()
        })
}
