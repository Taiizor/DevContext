use crate::error::{CoreError, Result};
use crate::storage::connection::DbConnection;
use crate::storage::sqlite_storage::SqliteStorage;
use std::path::Path;

/// Backup, restore, and integrity verification utilities for DevContext storage.
pub struct BackupService;

impl BackupService {
    /// Performs an online, non-blocking backup of the current database to `destination_path`.
    pub fn backup_to_file(
        storage: &SqliteStorage,
        destination_path: impl AsRef<Path>,
    ) -> Result<()> {
        let dest = destination_path.as_ref();
        if let Some(parent) = dest.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let mut dest_conn = DbConnection::open(dest)?;
        let src_arc = storage.connection_arc();
        let src_conn = src_arc
            .lock()
            .map_err(|e| CoreError::Concurrency(format!("Database lock poisoned: {}", e)))?;

        let backup = rusqlite::backup::Backup::new(&src_conn, &mut dest_conn)?;
        backup.run_to_completion(5, std::time::Duration::from_millis(50), None)?;

        Ok(())
    }

    /// Restores a database from `backup_path` to `target_db_path`.
    pub fn restore_from_file(
        backup_path: impl AsRef<Path>,
        target_db_path: impl AsRef<Path>,
    ) -> Result<SqliteStorage> {
        let backup_path = backup_path.as_ref();
        let target_path = target_db_path.as_ref();

        if !backup_path.exists() {
            return Err(CoreError::NotFound(format!(
                "Backup file not found at: {}",
                backup_path.display()
            )));
        }

        let mut target_conn = DbConnection::open(target_path)?;
        let backup_conn = DbConnection::open(backup_path)?;

        let backup = rusqlite::backup::Backup::new(&backup_conn, &mut target_conn)?;
        backup.run_to_completion(5, std::time::Duration::from_millis(50), None)?;

        SqliteStorage::open(target_path)
    }

    /// Runs SQLite `PRAGMA integrity_check` to verify database health.
    pub fn verify_integrity(storage: &SqliteStorage) -> Result<Vec<String>> {
        let src_arc = storage.connection_arc();
        let conn = src_arc
            .lock()
            .map_err(|e| CoreError::Concurrency(format!("Database lock poisoned: {}", e)))?;

        let mut stmt = conn.prepare("PRAGMA integrity_check;")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;

        let mut results = Vec::new();
        for row in rows {
            let res = row?;
            if res.to_lowercase() != "ok" {
                results.push(res);
            }
        }

        if results.is_empty() {
            Ok(vec!["ok".to_string()])
        } else {
            Err(CoreError::Integrity(format!(
                "Database integrity check failed: {:?}",
                results
            )))
        }
    }

    /// Runs SQLite `PRAGMA foreign_key_check` to ensure no orphan records exist.
    pub fn verify_foreign_keys(storage: &SqliteStorage) -> Result<()> {
        let src_arc = storage.connection_arc();
        let conn = src_arc
            .lock()
            .map_err(|e| CoreError::Concurrency(format!("Database lock poisoned: {}", e)))?;

        let mut stmt = conn.prepare("PRAGMA foreign_key_check;")?;
        let mut rows = stmt.query([])?;

        if rows.next()?.is_some() {
            return Err(CoreError::Integrity(
                "Foreign key constraint violations detected in database".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Project;
    use tempfile::tempdir;

    #[test]
    fn test_backup_restore_and_integrity() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("source.db");
        let backup_path = dir.path().join("backup.db");
        let restored_path = dir.path().join("restored.db");

        let storage = SqliteStorage::open(&db_path).unwrap();
        let project = Project::new("Backup Test", None).unwrap();
        storage.save_project(&project).unwrap();

        // Integrity check on fresh database
        let status = BackupService::verify_integrity(&storage).unwrap();
        assert_eq!(status, vec!["ok".to_string()]);
        BackupService::verify_foreign_keys(&storage).unwrap();

        // Perform backup
        BackupService::backup_to_file(&storage, &backup_path).unwrap();
        assert!(backup_path.exists());

        // Restore to fresh path
        let restored_storage =
            BackupService::restore_from_file(&backup_path, &restored_path).unwrap();
        let fetched = restored_storage.get_project(project.id).unwrap().unwrap();
        assert_eq!(fetched.name, "Backup Test");
    }
}
