use super::Migration;
use crate::error::Result;
use rusqlite::Transaction;

pub struct Migration0002Fts5Search;

impl Migration for Migration0002Fts5Search {
    fn version(&self) -> i64 {
        2
    }

    fn name(&self) -> &'static str {
        "0002_fts5_search"
    }

    fn up(&self, tx: &Transaction) -> Result<()> {
        tx.execute_batch(
            "
            -- 1. Create unified FTS5 virtual table
            CREATE VIRTUAL TABLE IF NOT EXISTS records_fts USING fts5(
                record_id UNINDEXED,
                project_id UNINDEXED,
                record_type UNINDEXED,
                title,
                body,
                tags,
                tokenize = 'unicode61 remove_diacritics 1'
            );

            -- 2. Populate FTS5 table from existing database records (where not deleted)
            -- Note: replace(..., 'ı', 'i') ensures Turkish dotless-i is normalized symmetrically
            -- with dotted-i and uppercase I in unicode61 without altering stored content.
            INSERT INTO records_fts(record_id, project_id, record_type, title, body, tags)
            SELECT r.id, r.project_id, r.record_type,
                   replace(r.title, 'ı', 'i'),
                   replace(n.content_markdown, 'ı', 'i'),
                   replace(r.tags, 'ı', 'i')
            FROM records r
            JOIN notes n ON r.id = n.record_id
            WHERE r.is_deleted = 0;

            INSERT INTO records_fts(record_id, project_id, record_type, title, body, tags)
            SELECT r.id, r.project_id, r.record_type,
                   replace(r.title, 'ı', 'i'),
                   replace(d.context || ' ' || d.decision || ' ' || d.consequences || ' ' || COALESCE(d.alternatives_considered, ''), 'ı', 'i'),
                   replace(r.tags, 'ı', 'i')
            FROM records r
            JOIN decisions d ON r.id = d.record_id
            WHERE r.is_deleted = 0;

            INSERT INTO records_fts(record_id, project_id, record_type, title, body, tags)
            SELECT r.id, r.project_id, r.record_type,
                   replace(r.title, 'ı', 'i'),
                   replace(COALESCE(m.description, ''), 'ı', 'i'),
                   replace(r.tags, 'ı', 'i')
            FROM records r
            JOIN milestones m ON r.id = m.record_id
            WHERE r.is_deleted = 0;

            INSERT INTO records_fts(record_id, project_id, record_type, title, body, tags)
            SELECT r.id, r.project_id, r.record_type,
                   replace(r.title, 'ı', 'i'),
                   replace(l.summary || ' ' || COALESCE(l.details, ''), 'ı', 'i'),
                   replace(r.tags, 'ı', 'i')
            FROM records r
            JOIN log_entries l ON r.id = l.record_id
            WHERE r.is_deleted = 0;

            INSERT INTO records_fts(record_id, project_id, record_type, title, body, tags)
            SELECT r.id, r.project_id, r.record_type,
                   replace(r.title, 'ı', 'i'),
                   replace(s.command_text || ' ' || COALESCE(s.explanation, ''), 'ı', 'i'),
                   replace(r.tags, 'ı', 'i')
            FROM records r
            JOIN saved_commands s ON r.id = s.record_id
            WHERE r.is_deleted = 0;

            -- 3. Synchronization Triggers

            -- Notes triggers
            CREATE TRIGGER IF NOT EXISTS trg_fts_notes_insert AFTER INSERT ON notes
            BEGIN
                DELETE FROM records_fts WHERE record_id = new.record_id;
                INSERT INTO records_fts(record_id, project_id, record_type, title, body, tags)
                SELECT r.id, r.project_id, r.record_type,
                       replace(r.title, 'ı', 'i'),
                       replace(new.content_markdown, 'ı', 'i'),
                       replace(r.tags, 'ı', 'i')
                FROM records r WHERE r.id = new.record_id AND r.is_deleted = 0;
            END;

            CREATE TRIGGER IF NOT EXISTS trg_fts_notes_update AFTER UPDATE ON notes
            BEGIN
                DELETE FROM records_fts WHERE record_id = new.record_id;
                INSERT INTO records_fts(record_id, project_id, record_type, title, body, tags)
                SELECT r.id, r.project_id, r.record_type,
                       replace(r.title, 'ı', 'i'),
                       replace(new.content_markdown, 'ı', 'i'),
                       replace(r.tags, 'ı', 'i')
                FROM records r WHERE r.id = new.record_id AND r.is_deleted = 0;
            END;

            -- Decisions triggers
            CREATE TRIGGER IF NOT EXISTS trg_fts_decisions_insert AFTER INSERT ON decisions
            BEGIN
                DELETE FROM records_fts WHERE record_id = new.record_id;
                INSERT INTO records_fts(record_id, project_id, record_type, title, body, tags)
                SELECT r.id, r.project_id, r.record_type,
                       replace(r.title, 'ı', 'i'),
                       replace(new.context || ' ' || new.decision || ' ' || new.consequences || ' ' || COALESCE(new.alternatives_considered, ''), 'ı', 'i'),
                       replace(r.tags, 'ı', 'i')
                FROM records r WHERE r.id = new.record_id AND r.is_deleted = 0;
            END;

            CREATE TRIGGER IF NOT EXISTS trg_fts_decisions_update AFTER UPDATE ON decisions
            BEGIN
                DELETE FROM records_fts WHERE record_id = new.record_id;
                INSERT INTO records_fts(record_id, project_id, record_type, title, body, tags)
                SELECT r.id, r.project_id, r.record_type,
                       replace(r.title, 'ı', 'i'),
                       replace(new.context || ' ' || new.decision || ' ' || new.consequences || ' ' || COALESCE(new.alternatives_considered, ''), 'ı', 'i'),
                       replace(r.tags, 'ı', 'i')
                FROM records r WHERE r.id = new.record_id AND r.is_deleted = 0;
            END;

            -- Milestones triggers
            CREATE TRIGGER IF NOT EXISTS trg_fts_milestones_insert AFTER INSERT ON milestones
            BEGIN
                DELETE FROM records_fts WHERE record_id = new.record_id;
                INSERT INTO records_fts(record_id, project_id, record_type, title, body, tags)
                SELECT r.id, r.project_id, r.record_type,
                       replace(r.title, 'ı', 'i'),
                       replace(COALESCE(new.description, ''), 'ı', 'i'),
                       replace(r.tags, 'ı', 'i')
                FROM records r WHERE r.id = new.record_id AND r.is_deleted = 0;
            END;

            CREATE TRIGGER IF NOT EXISTS trg_fts_milestones_update AFTER UPDATE ON milestones
            BEGIN
                DELETE FROM records_fts WHERE record_id = new.record_id;
                INSERT INTO records_fts(record_id, project_id, record_type, title, body, tags)
                SELECT r.id, r.project_id, r.record_type,
                       replace(r.title, 'ı', 'i'),
                       replace(COALESCE(new.description, ''), 'ı', 'i'),
                       replace(r.tags, 'ı', 'i')
                FROM records r WHERE r.id = new.record_id AND r.is_deleted = 0;
            END;

            -- Log entries triggers
            CREATE TRIGGER IF NOT EXISTS trg_fts_log_entries_insert AFTER INSERT ON log_entries
            BEGIN
                DELETE FROM records_fts WHERE record_id = new.record_id;
                INSERT INTO records_fts(record_id, project_id, record_type, title, body, tags)
                SELECT r.id, r.project_id, r.record_type,
                       replace(r.title, 'ı', 'i'),
                       replace(new.summary || ' ' || COALESCE(new.details, ''), 'ı', 'i'),
                       replace(r.tags, 'ı', 'i')
                FROM records r WHERE r.id = new.record_id AND r.is_deleted = 0;
            END;

            CREATE TRIGGER IF NOT EXISTS trg_fts_log_entries_update AFTER UPDATE ON log_entries
            BEGIN
                DELETE FROM records_fts WHERE record_id = new.record_id;
                INSERT INTO records_fts(record_id, project_id, record_type, title, body, tags)
                SELECT r.id, r.project_id, r.record_type,
                       replace(r.title, 'ı', 'i'),
                       replace(new.summary || ' ' || COALESCE(new.details, ''), 'ı', 'i'),
                       replace(r.tags, 'ı', 'i')
                FROM records r WHERE r.id = new.record_id AND r.is_deleted = 0;
            END;

            -- Saved commands triggers
            CREATE TRIGGER IF NOT EXISTS trg_fts_saved_commands_insert AFTER INSERT ON saved_commands
            BEGIN
                DELETE FROM records_fts WHERE record_id = new.record_id;
                INSERT INTO records_fts(record_id, project_id, record_type, title, body, tags)
                SELECT r.id, r.project_id, r.record_type,
                       replace(r.title, 'ı', 'i'),
                       replace(new.command_text || ' ' || COALESCE(new.explanation, ''), 'ı', 'i'),
                       replace(r.tags, 'ı', 'i')
                FROM records r WHERE r.id = new.record_id AND r.is_deleted = 0;
            END;

            CREATE TRIGGER IF NOT EXISTS trg_fts_saved_commands_update AFTER UPDATE ON saved_commands
            BEGIN
                DELETE FROM records_fts WHERE record_id = new.record_id;
                INSERT INTO records_fts(record_id, project_id, record_type, title, body, tags)
                SELECT r.id, r.project_id, r.record_type,
                       replace(r.title, 'ı', 'i'),
                       replace(new.command_text || ' ' || COALESCE(new.explanation, ''), 'ı', 'i'),
                       replace(r.tags, 'ı', 'i')
                FROM records r WHERE r.id = new.record_id AND r.is_deleted = 0;
            END;

            -- Records envelope triggers (title / tags update, soft-delete, restore)
            CREATE TRIGGER IF NOT EXISTS trg_fts_records_update AFTER UPDATE ON records
            BEGIN
                DELETE FROM records_fts WHERE record_id = old.id;
                INSERT INTO records_fts(record_id, project_id, record_type, title, body, tags)
                SELECT
                    new.id,
                    new.project_id,
                    new.record_type,
                    replace(new.title, 'ı', 'i'),
                    replace(
                        CASE new.record_type
                            WHEN 'note' THEN (SELECT content_markdown FROM notes WHERE record_id = new.id)
                            WHEN 'decision' THEN (SELECT context || ' ' || decision || ' ' || consequences || ' ' || COALESCE(alternatives_considered, '') FROM decisions WHERE record_id = new.id)
                            WHEN 'milestone' THEN (SELECT COALESCE(description, '') FROM milestones WHERE record_id = new.id)
                            WHEN 'log_entry' THEN (SELECT summary || ' ' || COALESCE(details, '') FROM log_entries WHERE record_id = new.id)
                            WHEN 'saved_command' THEN (SELECT command_text || ' ' || COALESCE(explanation, '') FROM saved_commands WHERE record_id = new.id)
                            ELSE ''
                        END,
                        'ı',
                        'i'
                    ),
                    replace(new.tags, 'ı', 'i')
                WHERE new.is_deleted = 0
                  AND (
                      EXISTS(SELECT 1 FROM notes WHERE record_id = new.id) OR
                      EXISTS(SELECT 1 FROM decisions WHERE record_id = new.id) OR
                      EXISTS(SELECT 1 FROM milestones WHERE record_id = new.id) OR
                      EXISTS(SELECT 1 FROM log_entries WHERE record_id = new.id) OR
                      EXISTS(SELECT 1 FROM saved_commands WHERE record_id = new.id)
                  );
            END;

            -- Hard delete trigger
            CREATE TRIGGER IF NOT EXISTS trg_fts_records_delete AFTER DELETE ON records
            BEGIN
                DELETE FROM records_fts WHERE record_id = old.id;
            END;
            ",
        )?;
        Ok(())
    }
}
