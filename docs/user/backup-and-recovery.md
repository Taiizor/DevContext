# Backup and Recovery Guide

DevContext treats developer knowledge as durable, human-owned project memory. Data safety and disaster recovery are built into core persistence workflows.

---

## 1. Storage Architecture

All DevContext project state is stored within the project repository:
```text
<project-root>/
└── .devcontext/
    ├── devcontext.db       # Primary SQLite database (WAL mode)
    ├── devcontext.db-wal   # Write-Ahead Log journal (during active writes)
    ├── devcontext.db-shm   # Shared memory index
    ├── backups/            # Local backup snapshot directory
    └── .gitignore          # Excludes SQLite database and WAL files from Git
```

---

## 2. Automatic Pre-Operation Backups

DevContext enforces automatic backup creation before any destructive operation:
1. **Pre-Import Backup:** Before applying changes from an imported JSON or ZIP bundle, an online atomic backup is created in `.devcontext/backups/`.
2. **Pre-Delete Backup:** Before executing `devcontext delete-data` (or the VS Code `Delete All Project Memory...` command), a full verified snapshot is created.

### Filename Uniqueness and Non-Overwriting Guarantee
- Backups use unique timestamped filenames with 128-bit UUID fragments:
  `backup-<prefix>-%Y%m%d-%H%M%S-<uuid>.db`
- Backups are created using atomic exclusive file creation (`create_new(true)`). An existing backup is **never silently overwritten**.
- If a filename collision occurs, the engine automatically retries with a fresh UUID.

---

## 3. Creating Manual Backups

### Via CLI
Run an atomic export to create an archive bundle:
```bash
# Export complete project memory to a portable ZIP archive
devcontext export --format zip --output backup-2026-10-31.zip

# Export machine-readable JSON bundle
devcontext export --format json --output context-backup.json
```

---

## 4. Disaster Recovery

### Restoring from an Automatic Backup
1. Stop running extension or CLI commands for the project.
2. Locate the desired backup file in `.devcontext/backups/` (e.g. `pre-delete-backup-20261031-120000-abcd.sqlite`).
3. Restore using the import command:
   ```bash
   # Inspect the backup before restoring
   devcontext inspect backup-file.sqlite
   ```
4. Or directly replace `.devcontext/devcontext.db`:
   ```bash
   cp .devcontext/backups/<backup-name>.sqlite .devcontext/devcontext.db
   devcontext doctor
   ```

### Emergency Safety Snapshots
If an online restore fails midway through stepping, DevContext executes an automatic rollback from a pre-restore safety snapshot:
- If automatic rollback also fails, the emergency safety snapshot is **never discarded or deleted**.
- It is preserved on disk at:
  `.devcontext/backups/emergency-manual-recovery-%Y%m%d-%H%M%S-<uuid>.db`
- The application reports the exact path and guides the user to manual recovery.
- Integrity can be verified with `devcontext doctor`.
