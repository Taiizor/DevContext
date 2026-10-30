use crate::error::{CoreError, Result};
use rusqlite::Connection;
use std::path::Path;

/// Configures and opens SQLite connections with strict invariants.
pub struct DbConnection;

impl DbConnection {
    /// Opens or creates an SQLite database at the specified path and configures PRAGMAs.
    pub fn open(path: impl AsRef<Path>) -> Result<Connection> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let conn = Connection::open(path)?;
        Self::apply_pragmas(&conn, true)?;
        Ok(conn)
    }

    /// Opens an in-memory SQLite database (primarily used for unit testing).
    pub fn open_in_memory() -> Result<Connection> {
        let conn = Connection::open_in_memory()?;
        Self::apply_pragmas(&conn, false)?;
        Ok(conn)
    }

    /// Applies required PRAGMAs per ADR 0008.
    fn apply_pragmas(conn: &Connection, enable_wal: bool) -> Result<()> {
        // Enforce foreign key constraints unconditionally
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        // 5-second busy timeout to handle concurrent access safely
        conn.busy_timeout(std::time::Duration::from_millis(5000))?;

        if enable_wal {
            // Write-Ahead Logging for high concurrency and crash resilience
            conn.pragma_update(None, "journal_mode", "WAL")?;
            conn.pragma_update(None, "synchronous", "NORMAL")?;
        }

        Self::verify_foreign_keys(conn)?;
        Ok(())
    }

    /// Verifies that foreign keys PRAGMA is indeed active.
    pub fn verify_foreign_keys(conn: &Connection) -> Result<()> {
        let fk_enabled: i32 = conn.query_row("PRAGMA foreign_keys;", [], |row| row.get(0))?;
        if fk_enabled != 1 {
            return Err(CoreError::Integrity(
                "SQLite foreign keys are not enabled".to_string(),
            ));
        }
        Ok(())
    }

    /// Returns the currently active journal mode (e.g. "wal" or "memory").
    pub fn get_journal_mode(conn: &Connection) -> Result<String> {
        let mode: String = conn.query_row("PRAGMA journal_mode;", [], |row| row.get(0))?;
        Ok(mode.to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_connection_pragmas() {
        let conn = DbConnection::open_in_memory().unwrap();
        DbConnection::verify_foreign_keys(&conn).unwrap();
        let mode = DbConnection::get_journal_mode(&conn).unwrap();
        assert_eq!(mode, "memory");

        // Verify FTS5 support
        conn.execute("CREATE VIRTUAL TABLE test_fts USING fts5(content);", [])
            .expect("SQLite FTS5 must be supported");
    }
}
