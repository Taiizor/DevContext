# DevContext --- MVP Acceptance Checklist

This checklist defines the explicit acceptance criteria required before the MVP (Phase 9) release can be declared ready. Each criterion is strictly verified against automated or documented manual tests.

---

## 1. Platform & Environment Compatibility

- [ ] **Windows 11 / 10:** CLI builds and passes integration tests natively.
- [ ] **Linux (Ubuntu / Debian / Fedora):** CLI builds and passes integration tests in CI.
- [ ] **macOS (Intel & Apple Silicon):** CLI builds and passes integration tests in CI.
- [ ] **Path Handling:** Functions correctly in directory paths containing spaces, non-ASCII characters, and deep nesting.
- [ ] **Read-Only Filesystem:** Displays an actionable error message without crashing when `.devcontext/` cannot be created or written to.
- [ ] **Offline Execution:** Operates completely without network connectivity or internet access.

---

## 2. Workspace & Project Identity

- [ ] **Git Repository Initialization:** `devcontext init` discovers git root, assigns UUIDv4, and creates `.devcontext/devcontext.db`.
- [ ] **Non-Git Workspace Initialization:** Successfully initializes and tracks context in folders that are not Git repositories.
- [ ] **Git Local Ignore Default:** On initialization in a Git repository, automatically recommends or configures `.git/info/exclude` to prevent accidental commit of local database.
- [ ] **Multi-Root Workspace Isolation:** Two distinct folders opened in VS Code or CLI maintain completely isolated SQLite databases and distinct project IDs.
- [ ] **Clone / Move Detection:** Moving a repository to another path preserves project identity and records; cloning prompts whether to link or separate context.

---

## 3. Notes & Decision Records (ADRs)

- [ ] **Note CRUD:** Create, read, update, list, and soft-delete/hard-delete notes via CLI and extension.
- [ ] **Note Categorization & Tags:** Add, remove, and filter by tags and categories.
- [ ] **Decision Lifecycle:** Record decisions in `proposed`, `accepted`, `rejected`, `deprecated`, or `superseded` states.
- [ ] **Superseding Decisions:** Superseding ADR preserves the original record with a direct foreign reference to the new ADR. Historical records remain immutable.
- [ ] **Markdown Fidelity:** Preserves raw Markdown formatting, headers, code blocks, and lists.

---

## 4. Milestones, Logs & Timeline

- [ ] **Milestones:** Create, update, and track milestone target dates and completion statuses.
- [ ] **Log Entries:** Add ad-hoc development events with custom timestamps.
- [ ] **Combined Timeline:** Chronological interleaving of DevContext records and Git commit metadata.
- [ ] **Provenance Distinction:** Git commits are visually and structurally differentiated from human-authored DevContext records.
- [ ] **Git Resiliency:** Timeline continues to work seamlessly on shallow clones, detached HEAD, or when Git is absent.

---

## 5. References

- [ ] **Reference Types:** Supports `file`, `folder`, `symbol`, `git_commit`, `git_branch`, `url`, `command`, and internal `record` links.
- [ ] **Relative Paths:** File and folder references store repository-relative paths to guarantee portability across different machines and clones.
- [ ] **Unavailable Target State:** Deleting or renaming a referenced file or commit marks the reference as "Unavailable" without corrupting or deleting the parent note.

---

## 6. Full-Text Search (FTS)

- [ ] **Search Coverage:** Queries match against record titles, Markdown contents, tags, and categories.
- [ ] **Offline FTS5:** Powered entirely by SQLite FTS5 index.
- [ ] **Search Filters:** Filters by record type, status, tag, and date range operate deterministically.
- [ ] **Snippets & Highlighting:** Search results return concise snippets showing matching query tokens.
- [ ] **Search Latency:** Executes within 300 ms for synthetic dataset of 10,000 records.

---

## 7. Command History & Safety

- [ ] **Manual Opt-In:** Saved commands are only persisted via intentional user action; no automatic terminal surveillance.
- [ ] **Secret Heuristics:** Warns the user when command text matches common API keys, tokens, or private key patterns.
- [ ] **No Automatic Execution:** Commands are strictly stored as inert text. Execution from UI or CLI requires explicit user confirmation.

---

## 8. Export, Import & Portability

- [ ] **Markdown Export:** Generates clean, standalone Markdown directory structure readable without DevContext.
- [ ] **JSON Export (v1):** Emits valid JSON conforming strictly to `schemas/export-v1.schema.json`.
- [ ] **Portable Archive (ZIP):** Bundles Markdown files and JSON schema into a portable archive.
- [ ] **Path Redaction:** Absolute local paths are sanitized from exports by default.
- [ ] **Pre-Import Backup:** Import operation creates an automated snapshot of existing database before applying changes.
- [ ] **Conflict Resolution:** Import detects conflicting UUIDs and provides deterministic policies: `skip`, `overwrite`, or `rename`.
- [ ] **Round-Trip Integrity:** Export followed by import in an empty workspace preserves exact records, tags, and timestamps.

---

## 9. Privacy, Security & Data Lifecycle

- [ ] **No Network / Telemetry:** Validated via proxy / network audit that zero outbound packets are sent.
- [ ] **No Environment Harvesting:** Process environment variables and `.env` files are never parsed or stored.
- [ ] **Data Destruction (`destroy`):** Irreversibly purges `.devcontext/` directory only after dual confirmation.
- [ ] **Database Inspection (`doctor`):** Runs integrity checks on SQLite database and reports schema migration version and file sizes.

---

## 10. VS Code Extension Experience

- [ ] **Lightweight Activation:** Extension activates only on DevContext commands or when `.devcontext/` is detected.
- [ ] **Non-Blocking UI:** Extension runs core invocations asynchronously without freezing editor rendering.
- [ ] **Keyboard-First:** All primary actions (new note, search, record decision) can be invoked via Command Palette and keyboard shortcuts.
- [ ] **Graceful Binary Resolution:** Clear notification and setup guidance if `devcontext` CLI binary is missing or incompatible.
