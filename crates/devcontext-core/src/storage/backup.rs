use crate::error::{CoreError, Result};
use crate::storage::connection::DbConnection;
use crate::storage::sqlite_storage::SqliteStorage;
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Backup, restore, and integrity verification utilities for DevContext storage.
pub struct BackupService;

impl BackupService {
    /// Performs an online, non-blocking backup of the current database to `destination_path`.
    /// Uses atomic exclusive creation (`create_new`) to eliminate TOCTOU races and guarantees
    /// that existing backup files are never silently overwritten.
    pub fn backup_to_file(
        storage: &SqliteStorage,
        destination_path: impl AsRef<Path>,
    ) -> Result<()> {
        let dest = destination_path.as_ref();

        if let Some(parent) = dest.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        // Atomic exclusive creation: fails if destination file already exists
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dest)
        {
            Ok(file) => drop(file),
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                return Err(CoreError::Storage(format!(
                    "Backup destination already exists and will not be overwritten: {}",
                    dest.display()
                )));
            }
            Err(err) => return Err(CoreError::Io(err)),
        }

        let mut dest_conn = DbConnection::open(dest)?;
        let src_arc = storage.connection_arc();
        let src_conn = src_arc
            .lock()
            .map_err(|e| CoreError::Concurrency(format!("Database lock poisoned: {}", e)))?;

        {
            let backup = rusqlite::backup::Backup::new(&src_conn, &mut dest_conn)?;
            backup.run_to_completion(5, std::time::Duration::from_millis(50), None)?;
        }

        drop(dest_conn);
        drop(src_conn);

        // Immediate integrity check on newly created backup file
        if let Err(e) = Self::verify_file_integrity(dest) {
            let _ = fs::remove_file(dest);
            return Err(e);
        }

        Ok(())
    }

    /// Atomically creates a unique backup file in `backup_dir` using a full 128-bit UUID and timestamp.
    /// Uses exclusive file creation (`create_new`) to avoid TOCTOU races.
    /// If a collision occurs, retries with a fresh filename without overwriting existing files.
    pub fn create_atomic_backup_file(
        storage: &SqliteStorage,
        backup_dir: impl AsRef<Path>,
        prefix: &str,
    ) -> Result<PathBuf> {
        let backup_dir = backup_dir.as_ref();
        if !backup_dir.exists() {
            fs::create_dir_all(backup_dir)?;
        }

        const MAX_RETRIES: usize = 10;
        let mut attempts = 0;

        loop {
            attempts += 1;
            let timestamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
            let full_uuid = Uuid::new_v4().to_string();
            let candidate_path =
                backup_dir.join(format!("{}-{}-{}.db", prefix, timestamp, full_uuid));

            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&candidate_path)
            {
                Ok(file) => {
                    drop(file);

                    let perform_res = (|| -> Result<()> {
                        let mut dest_conn = DbConnection::open(&candidate_path)?;
                        let src_arc = storage.connection_arc();
                        let src_conn = src_arc.lock().map_err(|e| {
                            CoreError::Concurrency(format!("Database lock poisoned: {}", e))
                        })?;

                        {
                            let backup = rusqlite::backup::Backup::new(&src_conn, &mut dest_conn)?;
                            backup.run_to_completion(
                                5,
                                std::time::Duration::from_millis(50),
                                None,
                            )?;
                        }

                        drop(dest_conn);
                        drop(src_conn);

                        Self::verify_file_integrity(&candidate_path)?;
                        Ok(())
                    })();

                    if let Err(e) = perform_res {
                        let _ = fs::remove_file(&candidate_path);
                        return Err(e);
                    }

                    return Ok(candidate_path);
                }
                Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                    if attempts >= MAX_RETRIES {
                        return Err(CoreError::Concurrency(format!(
                            "Failed to generate unique backup filename after {} attempts",
                            MAX_RETRIES
                        )));
                    }
                    continue;
                }
                Err(err) => return Err(CoreError::Io(err)),
            }
        }
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

        Self::verify_file_integrity(backup_path)?;

        let mut target_conn = DbConnection::open(target_path)?;
        let backup_conn = DbConnection::open(backup_path)?;

        {
            let backup = rusqlite::backup::Backup::new(&backup_conn, &mut target_conn)?;
            backup.run_to_completion(5, std::time::Duration::from_millis(50), None)?;
        }

        drop(target_conn);
        drop(backup_conn);
        SqliteStorage::open(target_path)
    }

    /// Safely restores database pages from `backup_path` into an active `SqliteStorage` instance.
    /// Employs a pre-restore safety snapshot to guarantee that if restore stepping fails,
    /// the destination database is automatically restored to its pre-restore state without corruption.
    pub fn restore_to_storage(
        backup_path: impl AsRef<Path>,
        storage: &SqliteStorage,
    ) -> Result<()> {
        Self::restore_to_storage_impl(backup_path, storage, false)
    }

    /// Test-only hook to inject a deterministic failure during restore stepping.
    #[doc(hidden)]
    pub fn restore_to_storage_with_injected_failure(
        backup_path: impl AsRef<Path>,
        storage: &SqliteStorage,
    ) -> Result<()> {
        Self::restore_to_storage_impl(backup_path, storage, true)
    }

    fn restore_to_storage_impl(
        backup_path: impl AsRef<Path>,
        storage: &SqliteStorage,
        inject_stepping_failure: bool,
    ) -> Result<()> {
        let backup_path = backup_path.as_ref();

        if !backup_path.exists() {
            return Err(CoreError::NotFound(format!(
                "Backup file not found at: {}",
                backup_path.display()
            )));
        }

        // Step 1: Pre-validate backup file integrity before touching destination
        Self::verify_file_integrity(backup_path)?;

        let backup_conn = DbConnection::open(backup_path)?;
        let dest_arc = storage.connection_arc();
        let mut dest_conn = dest_arc
            .lock()
            .map_err(|e| CoreError::Concurrency(format!("Database lock poisoned: {}", e)))?;

        // Step 2: Create a safety snapshot of the destination database before restore attempt
        let temp_dir = tempfile::tempdir()?;
        let safety_snapshot_path = temp_dir.path().join("pre_restore_safety.db");

        {
            let mut safety_conn = DbConnection::open(&safety_snapshot_path)?;
            let safety_backup = rusqlite::backup::Backup::new(&dest_conn, &mut safety_conn)?;
            safety_backup.run_to_completion(5, std::time::Duration::from_millis(50), None)?;
        }

        // Step 3: Attempt restore into destination connection
        let restore_step_res = (|| -> Result<()> {
            if inject_stepping_failure {
                return Err(CoreError::Storage(
                    "Injected restore stepping failure".to_string(),
                ));
            }

            let backup = rusqlite::backup::Backup::new(&backup_conn, &mut dest_conn)?;
            backup.run_to_completion(5, std::time::Duration::from_millis(50), None)?;
            Ok(())
        })();

        // Step 4: If restore failed, roll back destination database from safety snapshot
        if let Err(err) = restore_step_res {
            if safety_snapshot_path.exists() {
                if let Ok(safety_conn) = DbConnection::open(&safety_snapshot_path) {
                    if let Ok(rollback_backup) =
                        rusqlite::backup::Backup::new(&safety_conn, &mut dest_conn)
                    {
                        let _ = rollback_backup.run_to_completion(
                            5,
                            std::time::Duration::from_millis(50),
                            None,
                        );
                    }
                }
            }
            drop(dest_conn);
            drop(temp_dir);
            return Err(err);
        }

        drop(dest_conn);
        drop(backup_conn);
        drop(temp_dir);

        Self::verify_integrity(storage).map(|_| ())
    }

    /// Verifies the physical and logical integrity of an SQLite database file on disk.
    pub fn verify_file_integrity(path: impl AsRef<Path>) -> Result<()> {
        let p = path.as_ref();
        if !p.exists() {
            return Err(CoreError::NotFound(format!(
                "Database file not found for integrity check: {}",
                p.display()
            )));
        }

        let conn = DbConnection::open(p)?;
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
            Ok(())
        } else {
            Err(CoreError::Integrity(format!(
                "File integrity check failed for {}: {:?}",
                p.display(),
                results
            )))
        }
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
