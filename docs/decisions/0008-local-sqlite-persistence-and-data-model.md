# ADR 0008: Local SQLite persistence, schema migrations, and data model

- **Status:** Accepted
- **Date:** 2026-10-30

## Context

DevContext requires a durable, local-first storage engine that provides:
1. Fast, offline queries and full-text search.
2. Robust atomicity (ACID transactions) to prevent data corruption during unexpected OS shutdowns or crashes.
3. Reliable schema migrations over the lifecycle of the application.
4. Ease of backup, inspection, and portability.

## Decision

1. **Storage Engine:**
   - **SQLite 3** via the `rusqlite` crate (statically bundled with FTS5 enabled).
   - Write-Ahead Logging (`PRAGMA journal_mode = WAL;`) and synchronous normal (`PRAGMA synchronous = NORMAL;`) are enabled for high performance, high concurrency, and crash resilience.
   - Foreign key constraints are enforced unconditionally (`PRAGMA foreign_keys = ON;`).

2. **Timestamp Standard:**
   - All persisted timestamps use **UTC ISO-8601 / RFC 3339 strings with second or millisecond precision and explicit 'Z' suffix** (e.g., `2026-10-30T16:30:00Z`).
   - SQLite standard text comparison sorts ISO-8601 UTC strings lexicographically and chronologically with zero translation overhead.
   - Raw database inspection via standard SQLite viewers remains human-readable.

3. **Normalized Envelope + Typed Extension Tables:**
   Rather than an unvalidated EAV (Entity-Attribute-Value) model or a single giant untyped JSON column, we use a normalized **Envelope + Typed Detail** relational model:
   - `projects`: Project metadata, stable UUID, creation date, settings JSON.
   - `records`: Base envelope table containing common fields: `id` (UUID), `project_id`, `record_type`, `title`, `created_at`, `updated_at`, `tags` (JSON array or join table), and `is_deleted`.
   - `notes`: `record_id` (PK, FK -> records.id ON DELETE CASCADE), `content_markdown`, `category`, `is_pinned`, `is_archived`.
   - `decisions`: `record_id` (PK, FK -> records.id ON DELETE CASCADE), `status`, `context`, `decision`, `consequences`, `alternatives_considered`, `superseded_by` (FK -> records.id).
   - `milestones`: `record_id` (PK, FK -> records.id ON DELETE CASCADE), `status`, `target_date`, `completed_date`, `description`.
   - `log_entries`: `record_id` (PK, FK -> records.id ON DELETE CASCADE), `event_date`, `summary`, `details`.
   - `saved_commands`: `record_id` (PK, FK -> records.id ON DELETE CASCADE), `command_text`, `explanation`, `working_directory`.
   - `references`: `id` (UUID), `record_id` (FK -> records.id ON DELETE CASCADE), `ref_type`, `target`, `label`, `metadata_json`, `created_at`.
   - `records_fts`: SQLite FTS5 virtual table indexing `record_id`, `title`, `body_content`, `tags`.

4. **Schema Migrations:**
   - Migrations are versioned sequentially (`0001_initial_schema.sql`, `0002_...`).
   - Recorded in a `schema_migrations` table (`version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL, name TEXT NOT NULL`).
   - Every migration runs inside an explicit SQLite transaction (`BEGIN IMMEDIATE TRANSACTION ... COMMIT`). If any statement fails, the entire migration rolls back automatically.

## Consequences

### Positive
- Strict data integrity enforced by database foreign keys and transactions.
- Zero impedance mismatch between CLI queries and SQLite FTS5 indexing.
- Readily inspectable using standard SQLite GUI tools or CLI.

### Trade-offs
- Adding new record types requires adding a corresponding typed extension table and updating migrations. This ensures type safety and domain discipline at the cost of minor boilerplate.
