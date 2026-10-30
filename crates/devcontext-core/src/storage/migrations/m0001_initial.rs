use super::Migration;
use crate::error::Result;
use rusqlite::Transaction;

pub struct Migration0001Initial;

impl Migration for Migration0001Initial {
    fn version(&self) -> i64 {
        1
    }

    fn name(&self) -> &'static str {
        "0001_initial_schema"
    }

    fn up(&self, tx: &Transaction) -> Result<()> {
        tx.execute_batch(
            "
            -- Project workspace identity
            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL,
                vcs_identifier TEXT
            );

            -- Base metadata envelope for all DevContext records
            CREATE TABLE IF NOT EXISTS records (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                record_type TEXT NOT NULL,
                title TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                tags TEXT NOT NULL DEFAULT '[]',
                is_deleted INTEGER NOT NULL DEFAULT 0
            );

            CREATE INDEX IF NOT EXISTS idx_records_project_id ON records(project_id);
            CREATE INDEX IF NOT EXISTS idx_records_created_at ON records(created_at);
            CREATE INDEX IF NOT EXISTS idx_records_type ON records(record_type);

            -- Developer notes
            CREATE TABLE IF NOT EXISTS notes (
                record_id TEXT PRIMARY KEY REFERENCES records(id) ON DELETE CASCADE,
                content_markdown TEXT NOT NULL,
                category TEXT,
                is_pinned INTEGER NOT NULL DEFAULT 0,
                is_archived INTEGER NOT NULL DEFAULT 0
            );

            -- Architecture Decision Records (ADRs)
            CREATE TABLE IF NOT EXISTS decisions (
                record_id TEXT PRIMARY KEY REFERENCES records(id) ON DELETE CASCADE,
                status TEXT NOT NULL,
                context TEXT NOT NULL,
                decision TEXT NOT NULL,
                consequences TEXT NOT NULL,
                alternatives_considered TEXT,
                superseded_by TEXT REFERENCES records(id) ON DELETE SET NULL
            );

            -- Milestones
            CREATE TABLE IF NOT EXISTS milestones (
                record_id TEXT PRIMARY KEY REFERENCES records(id) ON DELETE CASCADE,
                status TEXT NOT NULL,
                target_date TEXT,
                completed_date TEXT,
                description TEXT
            );

            -- Development log entries
            CREATE TABLE IF NOT EXISTS log_entries (
                record_id TEXT PRIMARY KEY REFERENCES records(id) ON DELETE CASCADE,
                event_date TEXT NOT NULL,
                summary TEXT NOT NULL,
                details TEXT
            );

            -- Saved reusable commands (stored as text, never executed automatically)
            CREATE TABLE IF NOT EXISTS saved_commands (
                record_id TEXT PRIMARY KEY REFERENCES records(id) ON DELETE CASCADE,
                command_text TEXT NOT NULL,
                explanation TEXT,
                working_directory TEXT
            );

            -- Typed references linking records to files, symbols, commits, or URLs
            CREATE TABLE IF NOT EXISTS references_links (
                id TEXT PRIMARY KEY,
                record_id TEXT NOT NULL REFERENCES records(id) ON DELETE CASCADE,
                ref_type TEXT NOT NULL,
                target TEXT NOT NULL,
                label TEXT,
                metadata TEXT,
                created_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_references_record_id ON references_links(record_id);
            ",
        )?;

        Ok(())
    }
}
