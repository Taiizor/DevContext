use crate::error::{CoreError, Result};
use chrono::Utc;
use rusqlite::{Connection, Transaction};

pub mod m0001_initial;
pub mod m0002_fts5_search;

/// Represents a single versioned schema migration.
pub trait Migration {
    fn version(&self) -> i64;
    fn name(&self) -> &'static str;
    fn up(&self, tx: &Transaction) -> Result<()>;
}

/// Orchestrates database schema migrations.
pub struct MigrationRunner;

impl MigrationRunner {
    /// Returns all registered migrations in strict sequential order.
    pub fn all_migrations() -> Vec<Box<dyn Migration>> {
        vec![
            Box::new(m0001_initial::Migration0001Initial),
            Box::new(m0002_fts5_search::Migration0002Fts5Search),
        ]
    }

    /// Initializes the `schema_migrations` metadata table if not present.
    pub fn init_migration_table(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL,
                name TEXT NOT NULL
            );",
        )?;
        Ok(())
    }

    /// Returns the highest applied schema version, or 0 if no migrations have been applied.
    pub fn current_version(conn: &Connection) -> Result<i64> {
        Self::init_migration_table(conn)?;
        let mut stmt = conn.prepare("SELECT MAX(version) FROM schema_migrations;")?;
        let version: Option<i64> = stmt.query_row([], |row| row.get(0))?;
        Ok(version.unwrap_or(0))
    }

    /// Runs all pending migrations in an atomic transaction per migration.
    pub fn run_pending(conn: &mut Connection) -> Result<usize> {
        Self::init_migration_table(conn)?;
        let current = Self::current_version(conn)?;
        let migrations = Self::all_migrations();

        let mut applied_count = 0;
        for migration in migrations {
            if migration.version() > current {
                let tx = conn.transaction()?;
                migration.up(&tx)?;

                let now = Utc::now().to_rfc3339();
                tx.execute(
                    "INSERT INTO schema_migrations (version, applied_at, name) VALUES (?1, ?2, ?3);",
                    rusqlite::params![migration.version(), now, migration.name()],
                )?;

                tx.commit()?;
                applied_count += 1;
            }
        }

        Ok(applied_count)
    }

    /// Validates that all migrations recorded in the database are contiguous and recognized.
    pub fn verify_integrity(conn: &Connection) -> Result<()> {
        Self::init_migration_table(conn)?;
        let mut stmt =
            conn.prepare("SELECT version, name FROM schema_migrations ORDER BY version ASC;")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;

        let migrations = Self::all_migrations();

        for (expected_version, row) in (1..).zip(rows) {
            let (version, name) = row?;
            if version != expected_version {
                return Err(CoreError::Migration(format!(
                    "Non-contiguous migration detected. Expected version {}, found {}",
                    expected_version, version
                )));
            }

            let registered = migrations
                .iter()
                .find(|m| m.version() == version)
                .ok_or_else(|| {
                    CoreError::Migration(format!(
                        "Unknown migration version {} ({}) present in database",
                        version, name
                    ))
                })?;

            if registered.name() != name {
                return Err(CoreError::Migration(format!(
                    "Migration name mismatch for version {}: database has '{}', code has '{}'",
                    version,
                    name,
                    registered.name()
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::connection::DbConnection;

    #[test]
    fn test_migration_lifecycle() {
        let mut conn = DbConnection::open_in_memory().unwrap();
        assert_eq!(MigrationRunner::current_version(&conn).unwrap(), 0);

        let total = MigrationRunner::all_migrations().len();
        let applied = MigrationRunner::run_pending(&mut conn).unwrap();
        assert_eq!(applied, total);
        assert_eq!(
            MigrationRunner::current_version(&conn).unwrap(),
            total as i64
        );

        // Subsequent runs should apply 0 migrations
        let reapplied = MigrationRunner::run_pending(&mut conn).unwrap();
        assert_eq!(reapplied, 0);

        MigrationRunner::verify_integrity(&conn).unwrap();
    }
}
