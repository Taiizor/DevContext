use crate::domain::{
    Decision, DecisionStatus, LogEntry, Milestone, MilestoneStatus, Note, Project, Record,
    RecordType, Reference, ReferenceType, SavedCommand,
};
use crate::error::{CoreError, Result};
use crate::storage::connection::DbConnection;
use crate::storage::migrations::MigrationRunner;
use rusqlite::{params, Connection};
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Concrete SQLite persistence engine implementing domain storage operations.
#[derive(Clone)]
pub struct SqliteStorage {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteStorage {
    /// Opens an SQLite database file at `path`, configures PRAGMAs, and applies migrations.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let mut conn = DbConnection::open(path)?;
        MigrationRunner::run_pending(&mut conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Opens an isolated in-memory SQLite database and applies migrations (ideal for tests).
    pub fn open_in_memory() -> Result<Self> {
        let mut conn = DbConnection::open_in_memory()?;
        MigrationRunner::run_pending(&mut conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Executes a closure within an exclusive SQLite transaction.
    pub fn with_transaction<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&rusqlite::Transaction) -> Result<R>,
    {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Concurrency(format!("Database lock poisoned: {}", e)))?;
        let tx = conn.transaction()?;
        let res = f(&tx)?;
        tx.commit()?;
        Ok(res)
    }

    /// Clones the internal connection mutex for advanced storage operations (e.g. backup).
    pub(crate) fn connection_arc(&self) -> Arc<Mutex<Connection>> {
        self.conn.clone()
    }

    // ==========================================
    // Project Operations
    // ==========================================

    pub fn save_project(&self, project: &Project) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Concurrency(e.to_string()))?;
        conn.execute(
            "INSERT INTO projects (id, name, created_at, vcs_identifier)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                vcs_identifier = excluded.vcs_identifier;",
            params![
                project.id.to_string(),
                project.name,
                project.created_at,
                project.vcs_identifier
            ],
        )?;
        Ok(())
    }

    pub fn get_project(&self, id: Uuid) -> Result<Option<Project>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Concurrency(e.to_string()))?;
        let mut stmt = conn
            .prepare("SELECT id, name, created_at, vcs_identifier FROM projects WHERE id = ?1;")?;

        let mut rows = stmt.query(params![id.to_string()])?;
        if let Some(row) = rows.next()? {
            let id_str: String = row.get(0)?;
            let id = Uuid::parse_str(&id_str)
                .map_err(|e| CoreError::Integrity(format!("Invalid project UUID: {}", e)))?;
            let name: String = row.get(1)?;
            let created_at: String = row.get(2)?;
            let vcs_identifier: Option<String> = row.get(3)?;

            Ok(Some(Project::with_id(
                id,
                name,
                created_at,
                vcs_identifier,
            )?))
        } else {
            Ok(None)
        }
    }

    pub fn list_projects(&self) -> Result<Vec<Project>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Concurrency(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, name, created_at, vcs_identifier FROM projects ORDER BY created_at ASC;",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        })?;

        let mut projects = Vec::new();
        for r in rows {
            let (id_str, name, created_at, vcs_identifier) = r?;
            let id = Uuid::parse_str(&id_str)
                .map_err(|e| CoreError::Integrity(format!("Invalid project UUID: {}", e)))?;
            projects.push(Project::with_id(id, name, created_at, vcs_identifier)?);
        }

        Ok(projects)
    }

    // ==========================================
    // Internal Helper: Record Envelope
    // ==========================================

    fn save_record_envelope(tx: &rusqlite::Transaction, record: &Record) -> Result<()> {
        let tags_json = serde_json::to_string(&record.tags)?;
        tx.execute(
            "INSERT INTO records (id, project_id, record_type, title, created_at, updated_at, tags, is_deleted)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET
                project_id = excluded.project_id,
                record_type = excluded.record_type,
                title = excluded.title,
                created_at = excluded.created_at,
                updated_at = excluded.updated_at,
                tags = excluded.tags,
                is_deleted = excluded.is_deleted;",
            params![
                record.id.to_string(),
                record.project_id.to_string(),
                record.record_type.to_string(),
                record.title,
                record.created_at,
                record.updated_at,
                tags_json,
                if record.is_deleted { 1 } else { 0 },
            ],
        )?;
        Ok(())
    }

    fn read_record_envelope(row: &rusqlite::Row) -> Result<Record> {
        let id_str: String = row.get("id")?;
        let project_id_str: String = row.get("project_id")?;
        let record_type_str: String = row.get("record_type")?;
        let title: String = row.get("title")?;
        let created_at: String = row.get("created_at")?;
        let updated_at: String = row.get("updated_at")?;
        let tags_json: String = row.get("tags")?;
        let is_deleted_int: i32 = row.get("is_deleted")?;

        let id = Uuid::parse_str(&id_str)
            .map_err(|e| CoreError::Integrity(format!("Invalid record UUID: {}", e)))?;
        let project_id = Uuid::parse_str(&project_id_str)
            .map_err(|e| CoreError::Integrity(format!("Invalid project UUID: {}", e)))?;
        let record_type = RecordType::from_str(&record_type_str)?;
        let tags: Vec<String> = serde_json::from_str(&tags_json)?;

        Record::with_id(
            id,
            project_id,
            record_type,
            title,
            created_at,
            updated_at,
            tags,
            is_deleted_int == 1,
        )
    }

    // ==========================================
    // Note Operations
    // ==========================================

    pub fn save_note(&self, note: &Note) -> Result<()> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Concurrency(e.to_string()))?;
        let tx = conn.transaction()?;

        Self::save_record_envelope(&tx, &note.record)?;

        tx.execute(
            "INSERT INTO notes (record_id, content_markdown, category, is_pinned, is_archived)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(record_id) DO UPDATE SET
                content_markdown = excluded.content_markdown,
                category = excluded.category,
                is_pinned = excluded.is_pinned,
                is_archived = excluded.is_archived;",
            params![
                note.record.id.to_string(),
                note.content_markdown,
                note.category,
                if note.is_pinned { 1 } else { 0 },
                if note.is_archived { 1 } else { 0 },
            ],
        )?;

        tx.commit()?;
        Ok(())
    }

    pub fn get_note(&self, id: Uuid) -> Result<Option<Note>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Concurrency(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT r.id, r.project_id, r.record_type, r.title, r.created_at, r.updated_at,
                    r.tags, r.is_deleted, n.content_markdown, n.category, n.is_pinned, n.is_archived
             FROM records r
             JOIN notes n ON r.id = n.record_id
             WHERE r.id = ?1 AND r.is_deleted = 0;",
        )?;

        let mut rows = stmt.query(params![id.to_string()])?;
        if let Some(row) = rows.next()? {
            let record = Self::read_record_envelope(row)?;
            let content_markdown: String = row.get("content_markdown")?;
            let category: Option<String> = row.get("category")?;
            let is_pinned: i32 = row.get("is_pinned")?;
            let is_archived: i32 = row.get("is_archived")?;

            Ok(Some(Note {
                record,
                content_markdown,
                category,
                is_pinned: is_pinned == 1,
                is_archived: is_archived == 1,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_notes(&self, project_id: Uuid, include_archived: bool) -> Result<Vec<Note>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Concurrency(e.to_string()))?;
        let query = if include_archived {
            "SELECT r.id, r.project_id, r.record_type, r.title, r.created_at, r.updated_at,
                    r.tags, r.is_deleted, n.content_markdown, n.category, n.is_pinned, n.is_archived
             FROM records r
             JOIN notes n ON r.id = n.record_id
             WHERE r.project_id = ?1 AND r.is_deleted = 0
             ORDER BY n.is_pinned DESC, r.created_at DESC;"
        } else {
            "SELECT r.id, r.project_id, r.record_type, r.title, r.created_at, r.updated_at,
                    r.tags, r.is_deleted, n.content_markdown, n.category, n.is_pinned, n.is_archived
             FROM records r
             JOIN notes n ON r.id = n.record_id
             WHERE r.project_id = ?1 AND r.is_deleted = 0 AND n.is_archived = 0
             ORDER BY n.is_pinned DESC, r.created_at DESC;"
        };

        let mut stmt = conn.prepare(query)?;
        let mut rows = stmt.query(params![project_id.to_string()])?;

        let mut notes = Vec::new();
        while let Some(row) = rows.next()? {
            let record = Self::read_record_envelope(row)?;
            let content_markdown: String = row.get("content_markdown")?;
            let category: Option<String> = row.get("category")?;
            let is_pinned: i32 = row.get("is_pinned")?;
            let is_archived: i32 = row.get("is_archived")?;

            notes.push(Note {
                record,
                content_markdown,
                category,
                is_pinned: is_pinned == 1,
                is_archived: is_archived == 1,
            });
        }

        Ok(notes)
    }

    // ==========================================
    // Decision Operations
    // ==========================================

    pub fn save_decision(&self, decision: &Decision) -> Result<()> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Concurrency(e.to_string()))?;
        let tx = conn.transaction()?;

        Self::save_record_envelope(&tx, &decision.record)?;

        let superseded_str = decision.superseded_by.map(|id| id.to_string());
        tx.execute(
            "INSERT INTO decisions (record_id, status, context, decision, consequences, alternatives_considered, superseded_by)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(record_id) DO UPDATE SET
                status = excluded.status,
                context = excluded.context,
                decision = excluded.decision,
                consequences = excluded.consequences,
                alternatives_considered = excluded.alternatives_considered,
                superseded_by = excluded.superseded_by;",
            params![
                decision.record.id.to_string(),
                decision.status.to_string(),
                decision.context,
                decision.decision,
                decision.consequences,
                decision.alternatives_considered,
                superseded_str,
            ],
        )?;

        tx.commit()?;
        Ok(())
    }

    pub fn get_decision(&self, id: Uuid) -> Result<Option<Decision>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Concurrency(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT r.id, r.project_id, r.record_type, r.title, r.created_at, r.updated_at,
                    r.tags, r.is_deleted, d.status, d.context, d.decision, d.consequences,
                    d.alternatives_considered, d.superseded_by
             FROM records r
             JOIN decisions d ON r.id = d.record_id
             WHERE r.id = ?1 AND r.is_deleted = 0;",
        )?;

        let mut rows = stmt.query(params![id.to_string()])?;
        if let Some(row) = rows.next()? {
            let record = Self::read_record_envelope(row)?;
            let status_str: String = row.get("status")?;
            let status = DecisionStatus::from_str(&status_str)?;
            let context: String = row.get("context")?;
            let decision_text: String = row.get("decision")?;
            let consequences: String = row.get("consequences")?;
            let alternatives_considered: Option<String> = row.get("alternatives_considered")?;
            let superseded_by_str: Option<String> = row.get("superseded_by")?;
            let superseded_by = match superseded_by_str {
                Some(s) => Some(Uuid::parse_str(&s).map_err(|e| {
                    CoreError::Integrity(format!("Invalid superseded_by UUID: {}", e))
                })?),
                None => None,
            };

            Ok(Some(Decision {
                record,
                status,
                context,
                decision: decision_text,
                consequences,
                alternatives_considered,
                superseded_by,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_decisions(&self, project_id: Uuid) -> Result<Vec<Decision>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Concurrency(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT r.id, r.project_id, r.record_type, r.title, r.created_at, r.updated_at,
                    r.tags, r.is_deleted, d.status, d.context, d.decision, d.consequences,
                    d.alternatives_considered, d.superseded_by
             FROM records r
             JOIN decisions d ON r.id = d.record_id
             WHERE r.project_id = ?1 AND r.is_deleted = 0
             ORDER BY r.created_at DESC;",
        )?;

        let mut rows = stmt.query(params![project_id.to_string()])?;
        let mut decisions = Vec::new();
        while let Some(row) = rows.next()? {
            let record = Self::read_record_envelope(row)?;
            let status_str: String = row.get("status")?;
            let status = DecisionStatus::from_str(&status_str)?;
            let context: String = row.get("context")?;
            let decision_text: String = row.get("decision")?;
            let consequences: String = row.get("consequences")?;
            let alternatives_considered: Option<String> = row.get("alternatives_considered")?;
            let superseded_by_str: Option<String> = row.get("superseded_by")?;
            let superseded_by = match superseded_by_str {
                Some(s) => Some(Uuid::parse_str(&s).map_err(|e| {
                    CoreError::Integrity(format!("Invalid superseded_by UUID: {}", e))
                })?),
                None => None,
            };

            decisions.push(Decision {
                record,
                status,
                context,
                decision: decision_text,
                consequences,
                alternatives_considered,
                superseded_by,
            });
        }

        Ok(decisions)
    }

    // ==========================================
    // Milestone Operations
    // ==========================================

    pub fn save_milestone(&self, milestone: &Milestone) -> Result<()> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Concurrency(e.to_string()))?;
        let tx = conn.transaction()?;

        Self::save_record_envelope(&tx, &milestone.record)?;

        tx.execute(
            "INSERT INTO milestones (record_id, status, target_date, completed_date, description)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(record_id) DO UPDATE SET
                status = excluded.status,
                target_date = excluded.target_date,
                completed_date = excluded.completed_date,
                description = excluded.description;",
            params![
                milestone.record.id.to_string(),
                milestone.status.to_string(),
                milestone.target_date,
                milestone.completed_date,
                milestone.description,
            ],
        )?;

        tx.commit()?;
        Ok(())
    }

    pub fn get_milestone(&self, id: Uuid) -> Result<Option<Milestone>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Concurrency(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT r.id, r.project_id, r.record_type, r.title, r.created_at, r.updated_at,
                    r.tags, r.is_deleted, m.status, m.target_date, m.completed_date, m.description
             FROM records r
             JOIN milestones m ON r.id = m.record_id
             WHERE r.id = ?1 AND r.is_deleted = 0;",
        )?;

        let mut rows = stmt.query(params![id.to_string()])?;
        if let Some(row) = rows.next()? {
            let record = Self::read_record_envelope(row)?;
            let status_str: String = row.get("status")?;
            let status = MilestoneStatus::from_str(&status_str)?;
            let target_date: Option<String> = row.get("target_date")?;
            let completed_date: Option<String> = row.get("completed_date")?;
            let description: Option<String> = row.get("description")?;

            Ok(Some(Milestone {
                record,
                status,
                target_date,
                completed_date,
                description,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_milestones(&self, project_id: Uuid) -> Result<Vec<Milestone>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Concurrency(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT r.id, r.project_id, r.record_type, r.title, r.created_at, r.updated_at,
                    r.tags, r.is_deleted, m.status, m.target_date, m.completed_date, m.description
             FROM records r
             JOIN milestones m ON r.id = m.record_id
             WHERE r.project_id = ?1 AND r.is_deleted = 0
             ORDER BY r.created_at ASC;",
        )?;

        let mut rows = stmt.query(params![project_id.to_string()])?;
        let mut milestones = Vec::new();
        while let Some(row) = rows.next()? {
            let record = Self::read_record_envelope(row)?;
            let status_str: String = row.get("status")?;
            let status = MilestoneStatus::from_str(&status_str)?;
            let target_date: Option<String> = row.get("target_date")?;
            let completed_date: Option<String> = row.get("completed_date")?;
            let description: Option<String> = row.get("description")?;

            milestones.push(Milestone {
                record,
                status,
                target_date,
                completed_date,
                description,
            });
        }

        Ok(milestones)
    }

    // ==========================================
    // Log Entry Operations
    // ==========================================

    pub fn save_log_entry(&self, entry: &LogEntry) -> Result<()> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Concurrency(e.to_string()))?;
        let tx = conn.transaction()?;

        Self::save_record_envelope(&tx, &entry.record)?;

        tx.execute(
            "INSERT INTO log_entries (record_id, event_date, summary, details)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(record_id) DO UPDATE SET
                event_date = excluded.event_date,
                summary = excluded.summary,
                details = excluded.details;",
            params![
                entry.record.id.to_string(),
                entry.event_date,
                entry.summary,
                entry.details,
            ],
        )?;

        tx.commit()?;
        Ok(())
    }

    pub fn get_log_entry(&self, id: Uuid) -> Result<Option<LogEntry>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Concurrency(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT r.id, r.project_id, r.record_type, r.title, r.created_at, r.updated_at,
                    r.tags, r.is_deleted, l.event_date, l.summary, l.details
             FROM records r
             JOIN log_entries l ON r.id = l.record_id
             WHERE r.id = ?1 AND r.is_deleted = 0;",
        )?;

        let mut rows = stmt.query(params![id.to_string()])?;
        if let Some(row) = rows.next()? {
            let record = Self::read_record_envelope(row)?;
            let event_date: String = row.get("event_date")?;
            let summary: String = row.get("summary")?;
            let details: Option<String> = row.get("details")?;

            Ok(Some(LogEntry {
                record,
                event_date,
                summary,
                details,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_log_entries(&self, project_id: Uuid) -> Result<Vec<LogEntry>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Concurrency(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT r.id, r.project_id, r.record_type, r.title, r.created_at, r.updated_at,
                    r.tags, r.is_deleted, l.event_date, l.summary, l.details
             FROM records r
             JOIN log_entries l ON r.id = l.record_id
             WHERE r.project_id = ?1 AND r.is_deleted = 0
             ORDER BY l.event_date DESC;",
        )?;

        let mut rows = stmt.query(params![project_id.to_string()])?;
        let mut entries = Vec::new();
        while let Some(row) = rows.next()? {
            let record = Self::read_record_envelope(row)?;
            let event_date: String = row.get("event_date")?;
            let summary: String = row.get("summary")?;
            let details: Option<String> = row.get("details")?;

            entries.push(LogEntry {
                record,
                event_date,
                summary,
                details,
            });
        }

        Ok(entries)
    }

    // ==========================================
    // Saved Command Operations
    // ==========================================

    pub fn save_command(&self, cmd: &SavedCommand) -> Result<()> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Concurrency(e.to_string()))?;
        let tx = conn.transaction()?;

        Self::save_record_envelope(&tx, &cmd.record)?;

        tx.execute(
            "INSERT INTO saved_commands (record_id, command_text, explanation, working_directory)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(record_id) DO UPDATE SET
                command_text = excluded.command_text,
                explanation = excluded.explanation,
                working_directory = excluded.working_directory;",
            params![
                cmd.record.id.to_string(),
                cmd.command_text,
                cmd.explanation,
                cmd.working_directory,
            ],
        )?;

        tx.commit()?;
        Ok(())
    }

    pub fn get_command(&self, id: Uuid) -> Result<Option<SavedCommand>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Concurrency(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT r.id, r.project_id, r.record_type, r.title, r.created_at, r.updated_at,
                    r.tags, r.is_deleted, c.command_text, c.explanation, c.working_directory
             FROM records r
             JOIN saved_commands c ON r.id = c.record_id
             WHERE r.id = ?1 AND r.is_deleted = 0;",
        )?;

        let mut rows = stmt.query(params![id.to_string()])?;
        if let Some(row) = rows.next()? {
            let record = Self::read_record_envelope(row)?;
            let command_text: String = row.get("command_text")?;
            let explanation: Option<String> = row.get("explanation")?;
            let working_directory: Option<String> = row.get("working_directory")?;

            Ok(Some(SavedCommand {
                record,
                command_text,
                explanation,
                working_directory,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_commands(&self, project_id: Uuid) -> Result<Vec<SavedCommand>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Concurrency(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT r.id, r.project_id, r.record_type, r.title, r.created_at, r.updated_at,
                    r.tags, r.is_deleted, c.command_text, c.explanation, c.working_directory
             FROM records r
             JOIN saved_commands c ON r.id = c.record_id
             WHERE r.project_id = ?1 AND r.is_deleted = 0
             ORDER BY r.created_at DESC;",
        )?;

        let mut rows = stmt.query(params![project_id.to_string()])?;
        let mut commands = Vec::new();
        while let Some(row) = rows.next()? {
            let record = Self::read_record_envelope(row)?;
            let command_text: String = row.get("command_text")?;
            let explanation: Option<String> = row.get("explanation")?;
            let working_directory: Option<String> = row.get("working_directory")?;

            commands.push(SavedCommand {
                record,
                command_text,
                explanation,
                working_directory,
            });
        }

        Ok(commands)
    }

    // ==========================================
    // Generic Record Soft / Hard Delete
    // ==========================================

    pub fn soft_delete_record(&self, id: Uuid) -> Result<bool> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Concurrency(e.to_string()))?;
        let affected = conn.execute(
            "UPDATE records SET is_deleted = 1, updated_at = ?2 WHERE id = ?1 AND is_deleted = 0;",
            params![id.to_string(), chrono::Utc::now().to_rfc3339()],
        )?;
        Ok(affected > 0)
    }

    pub fn hard_delete_record(&self, id: Uuid) -> Result<bool> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Concurrency(e.to_string()))?;
        let affected = conn.execute(
            "DELETE FROM records WHERE id = ?1;",
            params![id.to_string()],
        )?;
        Ok(affected > 0)
    }

    // ==========================================
    // Reference Operations
    // ==========================================

    pub fn save_reference(&self, reference: &Reference) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Concurrency(e.to_string()))?;
        let metadata_json = match &reference.metadata {
            Some(v) => Some(serde_json::to_string(v)?),
            None => None,
        };

        conn.execute(
            "INSERT INTO references_links (id, record_id, ref_type, target, label, metadata, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
                ref_type = excluded.ref_type,
                target = excluded.target,
                label = excluded.label,
                metadata = excluded.metadata;",
            params![
                reference.id.to_string(),
                reference.record_id.to_string(),
                reference.ref_type.to_string(),
                reference.target,
                reference.label,
                metadata_json,
                reference.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_references_for_record(&self, record_id: Uuid) -> Result<Vec<Reference>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Concurrency(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, record_id, ref_type, target, label, metadata, created_at
             FROM references_links
             WHERE record_id = ?1
             ORDER BY created_at ASC;",
        )?;

        let mut rows = stmt.query(params![record_id.to_string()])?;
        let mut refs = Vec::new();
        while let Some(row) = rows.next()? {
            let id_str: String = row.get(0)?;
            let rec_id_str: String = row.get(1)?;
            let ref_type_str: String = row.get(2)?;
            let target: String = row.get(3)?;
            let label: Option<String> = row.get(4)?;
            let metadata_str: Option<String> = row.get(5)?;
            let created_at: String = row.get(6)?;

            let id = Uuid::parse_str(&id_str)
                .map_err(|e| CoreError::Integrity(format!("Invalid reference UUID: {}", e)))?;
            let r_id = Uuid::parse_str(&rec_id_str)
                .map_err(|e| CoreError::Integrity(format!("Invalid record UUID: {}", e)))?;
            let ref_type = ReferenceType::from_str(&ref_type_str)?;
            let metadata = match metadata_str {
                Some(s) => Some(serde_json::from_str(&s)?),
                None => None,
            };

            refs.push(Reference::with_id(
                id, r_id, ref_type, target, label, metadata, created_at,
            )?);
        }

        Ok(refs)
    }

    pub fn delete_reference(&self, id: Uuid) -> Result<bool> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CoreError::Concurrency(e.to_string()))?;
        let affected = conn.execute(
            "DELETE FROM references_links WHERE id = ?1;",
            params![id.to_string()],
        )?;
        Ok(affected > 0)
    }
}
