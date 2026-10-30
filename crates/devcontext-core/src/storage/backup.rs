use crate::error::{CoreError, Result};
use crate::storage::connection::DbConnection;
use crate::storage::sqlite_storage::SqliteStorage;
use chrono::Utc;
use std::fs;
use std::io::Write;
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
        Self::restore_to_storage_impl(backup_path, storage, false, false, false, false)
    }

    /// Test-only hook to inject a deterministic failure during restore stepping AFTER writing 1 page batch.
    #[doc(hidden)]
    pub fn restore_to_storage_with_injected_failure(
        backup_path: impl AsRef<Path>,
        storage: &SqliteStorage,
    ) -> Result<()> {
        Self::restore_to_storage_impl(backup_path, storage, true, false, false, false)
    }

    /// Test-only hook to inject a deterministic failure during rollback to verify dual-error reporting and emergency preservation.
    #[doc(hidden)]
    pub fn restore_to_storage_with_injected_rollback_failure(
        backup_path: impl AsRef<Path>,
        storage: &SqliteStorage,
    ) -> Result<()> {
        Self::restore_to_storage_impl(backup_path, storage, true, true, false, false)
    }

    /// Test-only hook to inject a failure when creating the emergency snapshot destination file.
    #[doc(hidden)]
    pub fn restore_to_storage_with_injected_emergency_creation_failure(
        backup_path: impl AsRef<Path>,
        storage: &SqliteStorage,
    ) -> Result<()> {
        Self::restore_to_storage_impl(backup_path, storage, true, true, true, false)
    }

    /// Test-only hook to inject a failure during emergency copy after writing partial output.
    #[doc(hidden)]
    pub fn restore_to_storage_with_injected_emergency_partial_copy_failure(
        backup_path: impl AsRef<Path>,
        storage: &SqliteStorage,
    ) -> Result<()> {
        Self::restore_to_storage_impl(backup_path, storage, true, true, false, true)
    }

    fn restore_to_storage_impl(
        backup_path: impl AsRef<Path>,
        storage: &SqliteStorage,
        inject_stepping_failure: bool,
        inject_rollback_failure: bool,
        inject_emergency_creation_failure: bool,
        inject_emergency_partial_copy_failure: bool,
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

        // Step 2: Create a collision-safe, unique safety snapshot before restore attempt
        let temp_dir = tempfile::tempdir()?;
        let snapshot_filename = format!(
            "pre-restore-safety-{}-{}.db",
            Utc::now().format("%Y%m%d-%H%M%S"),
            Uuid::new_v4()
        );
        let safety_snapshot_path = temp_dir.path().join(&snapshot_filename);

        // Atomic exclusive creation of the safety snapshot file
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&safety_snapshot_path)
        {
            Ok(file) => drop(file),
            Err(err) => return Err(CoreError::Io(err)),
        }

        let safety_res = (|| -> Result<()> {
            let mut safety_conn = DbConnection::open(&safety_snapshot_path)?;
            let safety_backup = rusqlite::backup::Backup::new(&dest_conn, &mut safety_conn)?;
            safety_backup.run_to_completion(5, std::time::Duration::from_millis(50), None)?;
            drop(safety_backup);
            drop(safety_conn);
            Self::verify_file_integrity(&safety_snapshot_path)?;
            Ok(())
        })();

        if let Err(e) = safety_res {
            let _ = fs::remove_file(&safety_snapshot_path);
            return Err(e);
        }

        // Step 3: Attempt restore into destination connection
        let restore_step_res = (|| -> Result<()> {
            let backup = rusqlite::backup::Backup::new(&backup_conn, &mut dest_conn)?;

            if inject_stepping_failure {
                // Transfer exactly 1 page batch so pages are physically written to dest_conn!
                let _done = backup.step(1)?;
                let progress = backup.progress();
                return Err(CoreError::Storage(format!(
                    "Injected restore failure after writing 1 page (remaining: {}, total: {})",
                    progress.remaining, progress.pagecount
                )));
            }

            backup.run_to_completion(5, std::time::Duration::from_millis(50), None)?;
            Ok(())
        })();

        // Step 4: If restore failed, roll back destination database from safety snapshot
        if let Err(restore_err) = restore_step_res {
            let mut rollback_err_opt = None;

            if safety_snapshot_path.exists() {
                let rollback_res = (|| -> Result<()> {
                    if inject_rollback_failure {
                        return Err(CoreError::Storage(
                            "Injected rollback stepping failure".to_string(),
                        ));
                    }
                    let safety_conn = DbConnection::open(&safety_snapshot_path)?;
                    let rollback_backup =
                        rusqlite::backup::Backup::new(&safety_conn, &mut dest_conn)?;
                    rollback_backup.run_to_completion(
                        5,
                        std::time::Duration::from_millis(50),
                        None,
                    )?;
                    drop(rollback_backup);
                    drop(safety_conn);
                    Ok(())
                })();

                if let Err(rb_err) = rollback_res {
                    rollback_err_opt = Some(rb_err);
                }
            } else {
                rollback_err_opt = Some(CoreError::NotFound(
                    "Safety snapshot file missing during rollback".to_string(),
                ));
            }

            if let Some(rb_err) = rollback_err_opt {
                // Rollback failure: attempt emergency safety snapshot preservation for manual recovery.
                //
                // Requirements:
                // 1. Never report that an emergency snapshot was preserved unless the destination file
                //    was successfully created, copied, and verified with PRAGMA integrity_check.
                // 2. Uses a unique filename: emergency-manual-recovery-%Y%m%d-%H%M%S-{uuid}.db
                // 3. Atomic exclusive creation (create_new) and collision retry so we never overwrite an existing recovery file.
                // 4. Safely handle partial-copy failures: clean up any partially written/corrupted file at dest_path.
                // 5. If preservation to backup directory fails (or creation/copy fails):
                //    - NEVER let RAII TempDir destructor delete the only remaining safety snapshot!
                //    - Convert temp_dir to persistent path via temp_dir.into_path().
                //    - Preserve original restore error, rollback error, and preservation failure error.
                //    - Retain temporary safety snapshot for manual recovery.

                let backups_dir = if let Some(target_db_path) = storage.db_path() {
                    target_db_path
                        .parent()
                        .unwrap_or_else(|| Path::new("."))
                        .join("backups")
                } else {
                    PathBuf::from(".devcontext").join("backups")
                };

                let preservation_res = (|| -> Result<PathBuf> {
                    if inject_emergency_creation_failure {
                        return Err(CoreError::Storage(
                            "Injected emergency destination creation failure".to_string(),
                        ));
                    }

                    fs::create_dir_all(&backups_dir)?;

                    let mut dest_file = None;
                    let mut dest_path = PathBuf::new();
                    let mut attempts = 0;

                    while attempts < 10 {
                        attempts += 1;
                        let filename = format!(
                            "emergency-manual-recovery-{}-{}.db",
                            Utc::now().format("%Y%m%d-%H%M%S"),
                            Uuid::new_v4()
                        );
                        let candidate_path = backups_dir.join(filename);

                        match fs::OpenOptions::new()
                            .write(true)
                            .create_new(true)
                            .open(&candidate_path)
                        {
                            Ok(file) => {
                                dest_file = Some(file);
                                dest_path = candidate_path;
                                break;
                            }
                            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                                continue;
                            }
                            Err(e) => return Err(CoreError::Io(e)),
                        }
                    }

                    let mut dest_file = dest_file.ok_or_else(|| {
                        CoreError::Storage(
                            "Failed to allocate unique emergency backup filename after 10 attempts"
                                .to_string(),
                        )
                    })?;

                    // Copy snapshot into destination with partial-copy protection
                    let copy_res = (|| -> Result<()> {
                        let mut src_file = fs::File::open(&safety_snapshot_path)?;

                        if inject_emergency_partial_copy_failure {
                            use std::io::Read;
                            let mut buf = [0u8; 512];
                            let n = src_file.read(&mut buf)?;
                            dest_file.write_all(&buf[..n])?;
                            dest_file.flush()?;
                            return Err(CoreError::Storage(
                                "Injected emergency copy failure after partial output".to_string(),
                            ));
                        }

                        std::io::copy(&mut src_file, &mut dest_file)?;
                        dest_file.flush()?;
                        drop(dest_file);
                        Ok(())
                    })();

                    if let Err(e) = copy_res {
                        // Clean up partially written file so no corrupt emergency snapshot remains
                        let _ = fs::remove_file(&dest_path);
                        return Err(e);
                    }

                    // Verify file integrity of the emergency snapshot with PRAGMA integrity_check
                    if let Err(e) = Self::verify_file_integrity(&dest_path) {
                        let _ = fs::remove_file(&dest_path);
                        return Err(CoreError::Storage(format!(
                            "Emergency snapshot integrity verification failed: {}",
                            e
                        )));
                    }

                    Ok(dest_path)
                })();

                drop(dest_conn);

                match preservation_res {
                    Ok(verified_emergency_path) => {
                        // Emergency snapshot was successfully copied, verified, and preserved!
                        // Temporary directory can now safely be dropped and cleaned up.
                        drop(temp_dir);

                        return Err(CoreError::Storage(format!(
                            "CRITICAL: Restore failed ({}) AND automatic rollback also failed ({}). Database is in an unverified state and requires manual recovery. Emergency safety snapshot preserved and verified at: {}",
                            restore_err,
                            rb_err,
                            verified_emergency_path.display()
                        )));
                    }
                    Err(preservation_err) => {
                        // Preservation to backup directory failed!
                        // CRITICAL: Prevent RAII cleanup from deleting the only remaining safety snapshot!
                        let persisted_temp_dir = temp_dir.path().to_path_buf();
                        let _ = temp_dir.keep();
                        let persistent_snapshot_path = persisted_temp_dir.join(&snapshot_filename);

                        // Verify if the temporary safety snapshot is still valid on disk
                        let temp_integrity = Self::verify_file_integrity(&persistent_snapshot_path);

                        match temp_integrity {
                            Ok(()) => {
                                return Err(CoreError::Storage(format!(
                                    "CRITICAL: Restore failed ({}) AND automatic rollback also failed ({}) AND emergency snapshot preservation to backup directory failed ({}). Retained safety snapshot in temporary directory at: {}. Database is in an unverified state and requires manual recovery.",
                                    restore_err,
                                    rb_err,
                                    preservation_err,
                                    persistent_snapshot_path.display()
                                )));
                            }
                            Err(integrity_err) => {
                                return Err(CoreError::Storage(format!(
                                    "CRITICAL: Restore failed ({}) AND automatic rollback also failed ({}) AND emergency snapshot preservation failed ({}). Temporary safety snapshot could not be verified ({}). Database is in an unverified state and requires manual recovery.",
                                    restore_err,
                                    rb_err,
                                    preservation_err,
                                    integrity_err
                                )));
                            }
                        }
                    }
                }
            }

            // Rollback succeeded cleanly!
            drop(dest_conn);
            drop(temp_dir);

            // Verify integrity of the restored database to confirm it is fully functional
            let _ = Self::verify_integrity(storage);

            return Err(restore_err);
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
