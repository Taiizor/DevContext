# Export, Import, and Data Safety Specification

## 1. Overview and Portability Guarantee

DevContext is built around developer ownership of project memory. To ensure long-term durability and avoid vendor lock-in, DevContext implements three complementary export representations:

1. **Versioned JSON (`export-v1.json`)**: Machine-readable format adhering strictly to `schemas/export-v1.schema.json` (schema version `1.0.0`).
2. **Human-Readable Markdown Directory**: Structured hierarchy of Markdown files with YAML frontmatter, readable in any text editor or static site generator without specialized tools.
3. **Portable ZIP Archive (`bundle.zip`)**: Self-contained archive combining `export-v1.json` at the archive root with the complete Markdown directory hierarchy.

DevContext guarantees that any valid `export-v1.json` or portable ZIP archive can be inspected and imported into another DevContext repository without data loss or network access.

---

## 2. Interchange Schema and Versioning

### 2.1 Versioning Policy
- Interchange schema versions follow Semantic Versioning (`MAJOR.MINOR.PATCH`).
- Current schema version is `1.0.0` defined in `schemas/export-v1.schema.json`.
- Deviations in `MAJOR` version (e.g. `2.0.0`) indicate incompatible structural changes and are strictly rejected with error code `INVALID_SCHEMA_VERSION`.
- Minor and patch version upgrades must remain backward compatible.

### 2.2 Record Graphs and Invariants
Export bundles contain:
- `project`: Project metadata, stable UUID, original name, and export timestamp (`exported_at`).
- `records`: List of all active and superseded records (Notes, Architecture Decisions, Milestones, Development Logs, Saved Commands).
- `references`: Associated file, folder, Git commit, web URL, or record reference links with sanitized target paths.

---

## 3. Path Sanitization and Privacy

### 3.1 In-Workspace Relativization
- Any absolute filesystem path within the workspace root is converted to a workspace-relative path prefixed with `./` (e.g. `/home/user/project/src/main.rs` becomes `./src/main.rs`).

### 3.2 Machine-Specific User Directory Redaction
- External paths outside the workspace have username and system drive/home prefixes redacted:
  - Windows: `C:/Users/<username>/...` is redacted to `~user/...`.
  - Unix / macOS: `/home/<username>/...` or `/Users/<username>/...` is redacted to `~user/...`.
- When explicit `--redact-paths` is specified, external paths are replaced with `[REDACTED_EXTERNAL_PATH]`.

---

## 4. Archive Security & Zip Slip Protection

Importing archives from untrusted sources entails directory traversal risks ("Zip Slip"). DevContext enforces safe archive handling:

1. **Path Normalization**: Replaces backslashes with forward slashes and strips leading drives and root slashes (`C:/`, `/`).
2. **Forbidden Components**: Any entry containing `..` or null bytes (`\0`) is immediately rejected with `ZIP_PATH_TRAVERSAL_REJECTED`.
3. **Canonical Sandbox Verification**: Before writing extracted files to disk, the destination path is resolved and verified to reside strictly within the temporary extraction sandbox directory.

---

## 5. Conflict Resolution Strategies

When importing bundles containing records whose UUIDs match existing records in the target project, the following conflict strategies are available:

- `skip`: Preserves the local existing record; skips the incoming record.
- `overwrite`: Replaces local content, title, tags, and metadata with the incoming record.
- `generate-new-id`: Generates a fresh UUID for the incoming record, importing it alongside the existing record.

---

## 6. Pre-Operation Backups and Data Safety

DevContext guarantees zero accidental data loss through transactional SQLite operations and automatic backup snapshots:

1. **Pre-Import Backup**: An online SQLite backup snapshot (`import-backup-<timestamp>.db`) is written to `.devcontext/backups/` before any non-dry-run import modifies records.
2. **Pre-Delete Backup**: `devcontext delete-data --force` writes a complete snapshot (`pre-delete-backup-<timestamp>.db`) before purging project tables.
3. **Data Inspection**: `devcontext inspect [file]` enables developers to audit incoming record counts, conflicting UUIDs, and schema compatibility prior to executing imports.
