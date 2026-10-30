# ADR 0011: SQLite FTS5 Full-Text Search and Reference Resolution Architecture

- **Status:** Accepted
- **Date:** 2026-10-30

## Context

Phase 5 requires establishing full-text search across all developer-authored DevContext records (notes, architecture decisions, milestones, logs, saved commands) and implementing reference resolution for linked project artifacts (Git commits, files, folders, URLs, symbols, records).

Two indexing strategies were evaluated:
1. **Multiple Per-Entity FTS5 Tables:**
   - *Pros:* Maps directly to each domain table (`notes_fts`, `decisions_fts`, etc.).
   - *Cons:* Cross-type search queries require cumbersome `UNION ALL` statements, BM25 scores cannot be calibrated reliably across separate tables, snippet generation logic must be duplicated, and project isolation requires filtering multiple disparate virtual tables.
2. **Unified `records_fts` FTS5 Virtual Table with Synchronization Triggers:**
   - *Pros:* Single virtual table indexing `record_id`, `project_id`, `record_type`, `title`, `body`, and `tags`. Enables atomic BM25 ranking across all record types, single-query type filtering, unified snippet generation, and straightforward project isolation via `project_id`. Synchronization triggers ensure zero stale data on inserts, updates, soft deletes, hard deletes, and restores.
   - *Cons:* Triggers must be maintained as database schema migrations.

Additionally, reference resolution must remain strictly local-first and read-only without initiating network requests, executing saved commands, or traversing outside the project boundary via symlinks, junctions, UNC paths, or parent directory components.

## Decision

1. **Unified FTS5 Table with Multilingual Tokenizer:**
   - DevContext establishes a unified virtual table:
     ```sql
     CREATE VIRTUAL TABLE records_fts USING fts5(
         record_id UNINDEXED,
         project_id UNINDEXED,
         record_type UNINDEXED,
         title,
         body,
         tags,
         tokenize = 'unicode61 remove_diacritics 1'
     );
     ```
   - `tokenize = 'unicode61 remove_diacritics 1'` provides full multilingual and Unicode support, removing diacritics for Latin-based languages (e.g. `ş -> s`, `ç -> c`, `ğ -> g`, `ü -> u`, `ö -> o`), handling punctuation gracefully, and enabling accent-insensitive matching.

2. **Turkish Search Normalization Strategy and Intentional Limitations:**
   - In SQLite FTS5 `unicode61`, standard Unicode case folding maps Latin capital `I` (U+0049) to `i` (U+0069), and `remove_diacritics 1` maps capital `İ` (U+0130) to `i`. However, Latin small dotless `ı` (U+0131) lacks diacritics and does not fold to `i` in Unicode default non-locale tables.
   - Consequently, without normalization, searching uppercase `ISPANAK` (using capital `I`) or `Isparta` would fail to match lowercase `ıspanak` or `ısparta`.
   - **Resolution:** DevContext normalizes dotless `ı` (U+0131) to `i` (U+0069) inside the derived `records_fts` virtual table (via SQLite `replace(..., 'ı', 'i')` in triggers and reindexing) and in incoming search query tokens (`query.replace('ı', "i")`).
   - **Stored Content Preservation:** All underlying domain tables (`records`, `notes`, `decisions`, `milestones`, `log_entries`, `saved_commands`) preserve the user's exact human-authored text, casing, and diacritics bit-for-bit without alteration. In `search_raw`, the title is selected from `records.title` rather than the normalized virtual table.
   - **Intentional Limitation:** Because `ı` and `i` are unified in the search index (analogous to `ş`/`s`, `ç`/`c`, `ğ`/`g`, `ü`/`u`, `ö`/`o` under `remove_diacritics 1`), words that differ solely by dotless `ı` vs dotted `i` (e.g., `ılık` and `ilik`) will match each other in search results. This ensures complete symmetric search behavior across Turkish keyboard layouts and casing without modifying persisted data.

3. **Trigger-Based Synchronization:**
   - All record changes are synchronized immediately within the same transactional boundary:
     - `AFTER INSERT` and `AFTER UPDATE` triggers on child tables (`notes`, `decisions`, `milestones`, `log_entries`, `saved_commands`) update `records_fts`.
     - `AFTER UPDATE` trigger on `records` synchronizes envelope updates (title, tags, soft delete via `is_deleted = 1`, and restore via `is_deleted = 0`).
     - `AFTER DELETE` trigger on `records` cascade-deletes search entries on hard deletion.
   - Soft-deleted records are immediately pruned from `records_fts` so searches never return deleted content.
   - Transaction rollbacks in SQLite automatically roll back FTS5 mutations atomically.

4. **BM25 Ranking and Deterministic Sorting:**
   - Queries use SQLite's native `bm25()` ranking function with column weightings:
     `bm25(records_fts, 0.0, 0.0, 0.0, 5.0, 1.0, 2.0)`
     giving `title` 5x weight, `tags` 2x weight, and `body` 1x weight.
   - Results are deterministically sorted by:
     `ORDER BY score ASC, r.created_at DESC, f.record_id ASC`.

5. **Resilient Query Sanitization:**
   - Queries are executed using parameterized statements.
   - If a user enters invalid FTS5 syntax (e.g. unbalanced quotes `"foo`, bare operators like `NOT`, `AND`, `*`), the search service intercepts the syntax error and gracefully retries with sanitized, quoted tokens.
   - Empty or punctuation-only queries return an empty result list without crashing or leaking raw SQLite error strings.

6. **Reference Resolution Boundaries and Path Traversal Security:**
   - References are resolved locally without side effects:
     - **`GitCommit` / `GitBranch`:** Verified using `GitAdapter`. If Git is not installed or the workspace is not a Git repo, status resolves gracefully to `Unavailable` without breaking the record.
     - **`File` / `Folder`:**
       - Lexical checks alone are rejected as insufficient.
       - The project root is canonicalized to establish the definitive security boundary.
       - **Existing Targets:** The target path is canonicalized using the filesystem driver (`canonicalize()`), resolving symbolic links, directory symlinks, and Windows NTFS junctions/reparse points. If the resolved canonical path falls outside the canonical project root, it is rejected as `Invalid`.
       - **Nonexistent Targets:** The resolver walks up parent directories until the lowest existing ancestor directory is found and canonicalized. If this existing ancestor resolves outside the canonical project root (e.g. via an external symlink, parent `..` components, or drive letters), the reference is rejected as `Invalid`. Only if the ancestor resides strictly within the project root is the target categorized as `TargetMissing`.
       - **Platform-Aware Comparisons:** Comparisons account for Windows drive-letter prefixes, UNC paths (`\\server\share`), and case insensitivity.
       - **Read-Only:** The resolver never modifies the filesystem or creates files/directories.
     - **`Url`:** Validated for syntactic correctness without sending network traffic.
     - **`Symbol` / `Command`:** Validated for non-empty identifiers/text. Commands are never executed automatically.
     - **`Record`:** Validated against the database to confirm the target record exists and is active.
   - Missing reference targets do not invalidate or delete the parent DevContext record.
   - Search results expose resolved references to facilitate record navigation.

## Consequences

### Positive
- Unified, fast search across all human-authored content.
- Native BM25 relevance ranking and contextual snippet generation.
- Zero stale index entries on any CRUD or soft/hard delete operation.
- Complete Turkish and Unicode diacritic support across all 7 character pairs (`I/ı`, `İ/i`, `Ş/ş`, `Ğ/ğ`, `Ü/ü`, `Ö/ö`, `Ç/ç`) with preserved stored content.
- Robust protection against path traversal through symlinks, Windows junctions, and nonexistent target probing.
- Fully offline and isolated reference resolution.
- Resilient against malformed search input.

### Trade-offs
- Slight write overhead during insertions and updates due to SQLite trigger execution.
- Diacritic-insensitive matching conflates dotless `ı` and dotted `i` in search indexes (e.g. `ılık` and `ilik`).
