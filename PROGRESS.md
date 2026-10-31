# DevContext --- Implementation Progress

> This file is the live execution tracker. Update it whenever work is
> started, completed, blocked, or materially changed.
>
> **Rule:** A checkbox may be checked only when the implementation
> exists and the stated verification evidence has been recorded.

## Project status

-   **Current phase:** Phase 9 --- Public MVP release (Prepared for Publication - Awaiting Approval Gates)
-   **Overall status:** Release artifacts built, verified, and packaged; awaiting publication authorization
-   **Last updated:** 2026-10-31
-   **Current milestone:** Public MVP Release v0.1.1
-   **Current blocker:** None recorded (Publication approval gates pending user authorization)

### Status legend

-   `[ ]` Not started
-   `[/]` In progress
-   `[x]` Complete and verified
-   `[!]` Blocked
-   `[-]` Deferred / intentionally out of scope

## Phase summary

  Phase                                       Status        Completion date   Notes
  ------------------------------------------- ------------- ----------------- --------------------------------------------------
  0\. Discovery and product contract          Complete      2026-10-30        PRD, architecture, ADRs 0001-0009, schema v1 ready
  1\. Repository and engineering foundation   Complete      2026-10-30        Workspace layout, Cargo, Bun/TS, CI, and test suite
  2\. Core domain and storage                 Complete      2026-10-30        SQLite WAL, schema v1 migrations, CRUD, backup/restore
  3\. CLI MVP                                 Complete      2026-10-30        init, status, doctor, note, decision, milestone, log commands
  4\. Git integration and timeline            Complete      2026-10-30        Read-only Git adapter, ADR 0010, unified timeline CLI
  5\. Search and references                   Complete      2026-10-30        SQLite FTS5 (BM25, unicode61), ReferenceResolver, ADR 0011
  6\. VS Code extension MVP                   Complete      2026-10-30        Manifest, CoreLocator, CoreClient IPC, TreeViews, QuickPick search, reference commands, ADR 0012, Extension Host + 62 TS tests
  7\. Export/import and data safety           Complete      2026-10-30        Versioned JSON, Markdown hierarchy, ZIP archives, inspection, conflict resolution, pre-operation backups, Zip Slip protections, CLI & VS Code UI
  8\. Quality and release readiness           Complete      2026-10-31        Multi-platform CI matrix, 10k search benchmark (<105ms), startup (<15ms), a11y tree items, release docs & RC checklist
  9\. Public MVP release                      In progress   ---               Artifacts packaged, docs & samples ready, awaiting publication approval gates

------------------------------------------------------------------------

## Active work

> Keep this section short. Move finished work to the completed log.

-   Phase 9 release artifacts (Windows x64 zip, VS Code VSIX, SHA256 checksums), documentation suite, sample project dataset, and community templates prepared. Awaiting user authorization for public publication approval gates.

## Blockers and risks

  ID    Blocker / risk                                                  Impact                                            Owner   Next action                                                                          Status
  ----- --------------------------------------------------------------- ------------------------------------------------- ------- ------------------------------------------------------------------------------------ --------
  R01   Extension binary delivery across platforms (Windows/macOS/Linux) Complexity in VS Code packaging and user install Core    Resolved in ADR 0012: Platform-specific VSIX packaging with layered fallback (Phase 8/9 CI matrix) Closed

## Phase 0 --- Discovery and product contract

-   [x] Confirm product name, repository name, license, and supported
    platforms.
-   [x] Define MVP and explicitly reject non-MVP features.
-   [x] Write user stories for initialization, notes, decisions, search,
    timeline, and export.
-   [x] Define privacy policy and collection boundaries.
-   [x] Decide default storage location and Git ignore behavior.
-   [x] Decide multi-root workspace behavior.
-   [x] Define stable project identity and clone detection behavior.
-   [x] Choose Rust/TypeScript workspace strategy.
-   [x] Define CLI-to-core and extension-to-core communication.
-   [x] Draft initial data model.
-   [x] Draft export schema version 1.
-   [x] Record major architecture decisions in `docs/decisions/`.

**Verification evidence:**
- Product Requirements Document created at `docs/requirements/product-requirements.md`.
- Architecture Overview created at `docs/architecture/overview.md`.
- MVP Acceptance Checklist created at `docs/requirements/mvp-acceptance-checklist.md`.
- Export Schema v1 created at `schemas/export-v1.schema.json` and validated with `bun -e "JSON.parse(Bun.file(...))"`.
- ADRs 0001 through 0009 documented in `docs/decisions/`.

## Phase 1 --- Repository and engineering foundation

-   [x] Create repository structure.
-   [x] Add root `AGENTS.md` and contribution guidance.
-   [x] Add license, README, SECURITY, CODE_OF_CONDUCT, and CONTRIBUTING
    documents.
-   [x] Configure formatting and linting for Rust and TypeScript.
-   [x] Configure CI for Windows, Linux, and macOS.
-   [x] Add dependency update policy.
-   [x] Add documented commit convention.
-   [x] Add issue and pull request templates.
-   [x] Add build scripts and developer setup instructions.
-   [x] Add test fixtures and sample repositories.
-   [x] Add release versioning strategy.
-   [x] Ensure no secrets or local database files are committed.

**Verification evidence:**
- Cargo workspace established with members `crates/devcontext-core` and `apps/cli`.
- Bun-based TypeScript package created for `apps/vscode-extension`.
- `LICENSE-MIT` and `LICENSE-APACHE` established for dual licensing.
- CI and templates configured. Baseline checks passing.

## Phase 2 --- Core domain and storage

-   [x] Define domain types and validation rules.
-   [x] Implement project initialization and stable identity.
-   [x] Implement SQLite connection management.
-   [x] Implement schema migrations.
-   [x] Implement transactional repository interfaces.
-   [x] Implement note CRUD.
-   [x] Implement decision CRUD.
-   [x] Implement milestone and log entry persistence.
-   [x] Implement tags and references.
-   [x] Implement saved command records (storage only).
-   [x] Implement data integrity checks.
-   [x] Implement database backup and restore primitives.
-   [x] Add migration tests.
-   [x] Add corruption and locked-database handling.
-   [x] Add concurrent access and interrupted-write tests.

**Verification evidence:**
- Domain models and SQLite persistence engine implemented with WAL mode and foreign key enforcement.
- Versioned migrations and backup/restore services implemented.
- 30 unit and integration tests passing.

## Phase 3 --- CLI MVP

-   [x] `devcontext init`
-   [x] `devcontext status`
-   [x] `devcontext note add`
-   [x] `devcontext note list`
-   [x] `devcontext note show`
-   [x] `devcontext note edit`
-   [x] `devcontext note delete`
-   [x] `devcontext decision add`
-   [x] `devcontext decision list`
-   [x] `devcontext decision show`
-   [x] `devcontext decision update`
-   [x] `devcontext milestone add`
-   [x] `devcontext log add`
-   [x] `devcontext doctor`
-   [x] CLI integration tests and command documentation.

**Verification evidence:**
- Implemented `WorkspaceManager` and thin CLI subcommands.
- Tested nested directory traversal, non-English unicode support, JSON validity, and project isolation.

## Phase 4 --- Git integration and timeline

-   [x] Detect Git repository root.
-   [x] Read current branch and HEAD.
-   [x] Read bounded commit metadata.
-   [x] Support configurable history depth.
-   [x] Link records to commits manually.
-   [x] Link current HEAD to a record.
-   [x] Assemble combined timeline.
-   [x] Distinguish Git events from authored records.
-   [x] Handle shallow clones, detached HEAD, no Git, and empty
    repositories.
-   [x] Keep Git integration read-only.
-   [x] Add Git fixture tests.

**Verification evidence:**
- Implemented dedicated, read-only `GitAdapter` in `devcontext-core` using `std::process::Command` (ADR 0010) with zero external C/crate dependencies.
- **Strict offline execution:** Configured all Git subprocesses with `GIT_NO_LAZY_FETCH=1`, `GIT_OPTIONAL_LOCKS=0`, and `GIT_TERMINAL_PROMPT=0` to guarantee offline execution, zero network requests in partial/blobless clones, and no lock file acquisition.
- **Missing executable resilience:** Handled `ErrorKind::NotFound` gracefully without crashes; timeline assembly continues serving DevContext records when Git is not installed or unavailable.
- **Shallow clone support:** Verified metadata retrieval in shallow clones (`git clone --depth 1`) without attempting to fetch parent objects.
- **Bounded retrieval & truncation discoverability:** Supported date constraints (`--since`, `--until`) are pushed directly down into the Git subprocess query. Invocations use a 1000-commit safety cap (`DEFAULT_MAX_COMMITS_UNBOUNDED = 1000`) to prevent memory exhaustion, with truncation made fully discoverable via `TimelineView.is_truncated` (JSON envelopes and CLI warning banners). The final global limit is applied only after merging all matching Git and DevContext events.
- **Structured delimiter parsing:** Uses ASCII record separator (`\x1e`, 0x1E) and unit separator (`\x1f`, 0x1F) to parse commit messages with multiline content, quotes, commas, brackets, and UTF-8 emojis (`🚀`, `✨`, `⚡`) deterministically. Empty commit messages fall back to short hash, and atypical authors (missing email/name) are handled safely.
- **Predictable date filtering:** Implemented timestamp and date-only parsing in `TimelineFilter` where `--until=YYYY-MM-DD` covers the entire calendar day up to 23:59:59.999Z.
- **ADR 0010 update:** Accurately distinguished command allowlist guarantees, `GIT_OPTIONAL_LOCKS=0` scope, and subprocess execution limitations in `docs/decisions/0010-process-based-read-only-git-adapter.md`.
- **54 total workspace tests passing:** `cargo test --workspace` passes cleanly with 0 failures across unit and integration tests (10 CLI + 44 Core).
- **Code hygiene:** `cargo fmt --check` and `cargo clippy --all-targets --all-features -- -D warnings` pass with 0 warnings.
- **TypeScript checks:** `bun run typecheck` and `bun test` pass cleanly.

## Phase 5 --- Search and references

-   [x] Implement full-text indexing.
-   [x] Index titles, bodies, tags, and selected metadata.
-   [x] Implement snippets and ranking.
-   [x] Add record type, status, tag, and date filters.
-   [x] Implement file and folder references.
-   [x] Implement commit and branch references.
-   [x] Implement URL references.
-   [x] Implement command references.
-   [x] Implement unavailable-target states.
-   [x] Add reindex command.
-   [x] Add search performance tests.
-   [x] Document search behavior.

**Verification evidence:**
- **SQLite FTS5 virtual table & migration:** Created migration `m0002_fts5_search.rs` registering unified `records_fts` table using `tokenize = 'unicode61 remove_diacritics 1'`. Automatic synchronization triggers maintain consistency across INSERT, UPDATE, soft-delete (`is_deleted = 1`), hard-delete, and restore operations.
- **Turkish search normalization & stored content preservation:** Addressed SQLite `unicode61`'s non-locale case folding limitation where dotless `ı` (U+0131) does not fold to `i`. Implemented symmetric normalization mapping `ı` to `i` in the search index (`records_fts`) and query tokens. Original stored content across all domain tables (`records`, `notes`, `decisions`, etc.) is strictly preserved bit-for-bit without alteration. Symmetrically tested across all 7 Turkish character pairs: `I/ı`, `İ/i`, `Ş/ş`, `Ğ/ğ`, `Ü/ü`, `Ö/ö`, and `Ç/ç`.
- **Search service & ranking:** Implemented `SearchService`, `SearchResult`, and `SearchFilter` with SQLite `bm25(records_fts, 5.0, 1.0, 2.0)` weighting (title 5x, tags 2x, body 1x). Contextual snippets generated with `snippet(records_fts, 1, '[match]', '[/match]', '...', 12)`.
- **Query resilience:** Unbalanced quotes, wildcards, or invalid FTS5 operators trigger an automatic tokenized phrase fallback (`sanitize_fts5_query`) rather than surfacing raw SQLite syntax errors.
- **Canonical reference path traversal security:** Implemented `ReferenceResolver` with canonical filesystem resolution. Verified existing targets against canonical root (rejecting external symlinks, junctions, reparse points, UNC, and absolute paths as `Invalid`). Nonexistent targets walk up parent directories to canonicalize the deepest existing ancestor, rejecting escaped ancestors as `Invalid` while categorizing internal missing targets as `TargetMissing`.
- **CLI search workflows:** Added `devcontext search <query>` with `--type`, `--limit`, `--reindex`, and `--json` support complying with ADR 0007 response envelope.
- **ADR 0011 established & updated:** Documented design, Turkish normalization rationale/trade-offs, and reference path traversal canonical resolution in `docs/decisions/0011-sqlite-fts5-search-and-reference-resolution.md`.
- **Automated test suite (72 Rust tests):** 72 total Rust tests passing across the workspace (11 CLI + 61 Core), including 10 dedicated search tests (`search_test.rs`) verifying CRUD sync, soft-delete filtering, Unicode diacritic matching, Turkish casing/casing symmetry, syntax safety, BM25 weighting, and reindex performance (<50ms for 100 records), 7 reference tests (`reference_test.rs`) covering canonical traversal, NTFS junctions, external symlinks, and URL/commit/symbol resolution, and comprehensive CLI integration tests (`cli_search_test.rs`).
- **Code hygiene:** `cargo fmt --check` and `cargo clippy --all-targets --all-features -- -D warnings` pass with 0 warnings. Bun TypeScript checks (`bun run typecheck` and `bun test`) pass cleanly.

## Phase 6 --- VS Code extension MVP

-   [x] Create extension manifest and activation strategy.
-   [x] Detect workspace roots and multi-root workspaces.
-   [x] Locate or provision compatible core executable.
-   [x] Implement extension/core version handshake.
-   [x] Add project initialization command.
-   [x] Add activity bar container and project overview.
-   [x] Add notes and decisions tree views.
-   [x] Add timeline view.
-   [x] Add search command and quick-pick results.
-   [x] Add note creation/editing UI.
-   [x] Add decision creation/editing UI.
-   [x] Add milestone and log commands.
-   [x] Add file and selection reference actions.
-   [x] Add reference-opening actions.
-   [x] Add export/import preview and confirmation.
-   [x] Add privacy and storage settings.
-   [x] Handle missing/incompatible core executable.
-   [x] Add extension host tests and manual test checklist.
-   [x] Ensure long operations do not block the UI.

**Verification evidence:**
- **Extension Manifest & Bundling (`apps/vscode-extension/package.json`):** Configured activation events (`onStartupFinished`, `workspaceContains:.devcontext/devcontext.db`), Activity Bar view container (`devcontext-container`), 4 custom tree views (`devcontext.views.overview`, `devcontext.views.notes`, `devcontext.views.decisions`, `devcontext.views.timeline`), 24 user-facing commands, menus (editor context, view title, inline tree items), keyboard shortcuts, and configuration settings (`devcontext.core.executablePath`, `devcontext.core.timeoutMs`, `devcontext.privacy.redactPathsInExport`, `devcontext.search.debounceMs`, `devcontext.timeline.maxEvents`). Bundled with `bun build src/extension.ts --outfile dist/extension.js --target node --external vscode` (65.42 KB single-file bundle).
- **Core Locator & Version Handshake (`src/core/locator.ts`, `src/core/handshake.ts`, ADR 0012):** Implemented 3-tier resolution strategy (`devcontext.core.executablePath` -> bundled platform-specific VSIX `bin/<platform>-<arch>/devcontext` -> system `PATH`) with executable permissions check. Strict version handshake verifies CLI `devcontext --version` and compatibility with schema/protocol `1.x`. Missing/incompatible binaries degrade gracefully with user-actionable prompts and status bar warnings without blocking VS Code.
- **IPC Protocol Client (`src/core/client.ts`):** `CoreClient` runs the core binary with structured arguments and `--json` flag per ADR 0007. No shell interpolation, strict timeouts, VS Code cancellation token integration (`AbortSignal`), and typed envelope validation (`ResponseEnvelope<T>`).
- **Multi-Root Workspace Isolation (`src/workspace/workspaceManager.ts`):** Independent discovery across workspace roots without silent merging. Safe active-folder detection, read-only workspace protections (preventing init/write operations on untrusted workspaces), and change notifications.
- **Tree Views & Status Bar (`src/views/`, `src/statusbar/statusBar.ts`):**
  - `OverviewTreeDataProvider`: Shows project status, record statistics, DB health, or guided setup when uninitialized.
  - `NotesTreeDataProvider`: Renders notes categorized by status (pinned, active, archived), formatted tags, and click-to-preview.
  - `DecisionsTreeDataProvider`: Categorizes ADRs with status badges (`[ACCEPTED]`, `[PROPOSED]`, `[SUPERSEDED]`, `[REJECTED]`).
  - `TimelineTreeDataProvider`: Chronological stream merging Git and DevContext events with type filtering.
  - `DevContextStatusBar`: Real-time status in VS Code status bar showing active project name, counts tooltip, and warning states.
- **Commands & Interactive Workflows (`src/commands/`):**
  - Note management: `newNote`, `newNoteFromSelection` (capturing editor range/relative file reference), `showNote`, `editNote`, `deleteNote`.
  - Decision management: `newDecision` (interactive multi-step prompts for title, context, decision, consequences), `showDecision`, `supersedeDecision`.
  - Milestones & logs: `newMilestone`, `completeMilestone`, `newLog`.
  - Live search: `devcontext.search` with debounced live QuickPick, BM25 ranking, highlight snippets, and direct item opening.
  - References: `openReference` safely opening line ranges, workspace folders, external URLs, or Git commit hashes; `linkFileToNote` linking active files.
  - Export/Import: `devcontext.export` and `devcontext.import` presenting detailed preview modals (record counts, privacy redaction summary) and requiring explicit user confirmation before selecting folders or executing.
  - Doctor: `devcontext.doctor` outputting system and storage diagnostics directly to a dedicated OutputChannel.
- **Automated Test Suites (62 Bun tests + Real Extension Host + 72 Rust tests):**
  - Bun test suite (`cd apps/vscode-extension && bun test`): 62 passing tests across 10 suites (`client_and_ipc.test.ts`, `commands_and_workflows.test.ts`, `export_import_preview.test.ts`, `extension.test.ts`, `git_commit_reference.test.ts`, `locator_and_handshake.test.ts`, `views_and_providers.test.ts`, `workspace_and_isolation.test.ts`, `real_core_integration.test.ts`, `real_workflow_e2e.test.ts`).
  - Real Extension Host test suite (`bun run test:integration` via `@vscode/test-electron`): Launched local VS Code Extension Development Host, verified extension identity (`devcontext.devcontext`), all 24 commands registered and active, reference actions (file with line range anchor, external URL, real git commit opening syntax-colored diff editor in an isolated 2-commit git repository, unavailable and invalid commit SHAs, missing target handling), tree view resolution, and clean deactivation.
  - TypeScript typecheck (`bun run typecheck`): 0 errors (`tsc --noEmit`).
  - Rust workspace test suite (`cargo test --workspace`): 72 passing tests across all crates (61 core + 11 CLI).
  - Rust formatting & clippy (`cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`): 0 warnings.
  - VSIX packaging (`bun run package`): Generated clean, production-ready `devcontext-0.1.0.vsix` containing 7 files (manifest, compiled `dist/extension.js`, license, readme, icon; 0 dev dependencies, 0 test files).
- **ADR 0012:** Documented Rust Core Binary Distribution & Resolution Strategy at `docs/decisions/0012-rust-core-binary-distribution-and-resolution-strategy.md`.

## Phase 7 --- Export/import and data safety

-   [x] Finalize versioned JSON schema.
-   [x] Implement Markdown export.
-   [x] Implement portable archive export.
-   [x] Add export preview.
-   [x] Add path redaction.
-   [x] Add import preview and conflict resolution.
-   [x] Add import schema upgrades.
-   [x] Back up before destructive imports.
-   [x] Add data deletion command and UI.
-   [x] Add data inspection command.
-   [x] Add round-trip tests.
-   [x] Test malformed and large archives.
-   [x] Document compatibility guarantees.

**Verification evidence:**
- **Versioned JSON & Markdown Hierarchy (`crates/devcontext-core/src/export/`):**
  - Typed interchange data model conforming to `schemas/export-v1.schema.json` (schema 1.0.0).
  - Schema upgrade path via `upgrade_bundle`: deterministically supports schema `1.0.0`, upgrades older supported pre-release schema `0.9.0` preserving IDs, timestamps, references, and payloads, and safely rejects unsupported newer major versions (e.g. `2.0.0`).
  - Human-readable Markdown export directory structure (`decisions/`, `notes/`, `milestones/`, `logs/`, `commands/`, `README.md`) with YAML frontmatter metadata and generated table of contents.
  - Portable ZIP archives packaging `export-v1.json` at root alongside Markdown hierarchy.
- **Privacy & Security (`sanitization.rs` & `archive.rs`):**
  - Workspace-relative path relativization (`./src/main.rs`).
  - Cross-platform machine-specific user directory redaction (`~user/...` / `<redacted>`) across references and saved command working directories, plus explicit `--redact-paths` sanitization (`[REDACTED_EXTERNAL_PATH]`).
  - Strict Zip Slip traversal protection: rejects `..` path segments, absolute archive paths, null bytes (`\0`), Windows Alternate Data Streams (`:`), UNC paths, and symlink archive entries (`unix_mode & 0o120000`).
  - Archive resource limits: enforces max archive size (100 MB), max entries (10,000), max uncompressed entry size (50 MB), max total uncompressed size (200 MB), max compression ratio (100x zip-bomb guard), and max JSON payload size (25 MB). Cleans up temporary sandbox on extraction failure.
- **Data Safety, Conflict Resolution & Recovery:**
  - Complete conflict resolution policies: `skip`, `overwrite` / `replace` (with orphaned reference wiping), `merge` (tag union, deduplicated reference union, timestamp-based field updates), and `generate-new-id`.
  - Atomicity and rollback recovery: pre-import SQLite backup snapshot created in `.devcontext/backups/import-backup-<timestamp>.db` before modifying records; failed/aborted imports automatically restore database from backup.
  - Complete project data deletion (`devcontext delete-data --force`) with automatic pre-delete snapshot (`pre-delete-backup-<timestamp>.db`).
  - Offline inspection command (`devcontext inspect [file]`) auditing record counts and conflicts prior to import.
- **CLI Subcommands (`apps/cli/src/commands/`):**
  - `devcontext export [--format <json|markdown|zip>] [--out <path>] [--preview] [--redact-paths] [--json]`
  - `devcontext import <file> [--on-conflict <skip|overwrite|replace|merge|generate-new-id>] [--dry-run] [--json]`
  - `devcontext inspect [file] [--json]`
  - `devcontext delete-data [--force] [--json]`
- **VS Code Extension Integration (`apps/vscode-extension/`):**
  - `devcontext.export`: QuickPick format selector, interactive preview modal detailing record counts and path redaction warning, and user confirmation.
  - `devcontext.import`: Open dialog, safety guarantee confirmation, conflict policy selection with explicit overwrite warning, and client import execution.
  - `devcontext.inspect`: QuickPick audit modal displaying bundle record counts and schema version.
  - `devcontext.deleteData`: Dual-confirmation destructive safety dialog with automatic backup assurance.
- **Automated Verification:**
  - `cargo test -p devcontext-core --test export_import_test`: 15 integration tests passing (including schema upgrades, future version rejection, merge semantics, replace reference cleanup, zip bomb protection, ADS/null-byte rejection, and atomic rollback on failure).
  - `cargo test -p devcontext-cli --test cli_export_import_test`: 1 CLI integration test passing.
  - `cargo test --workspace`: 92 total Rust tests passing across core and CLI with 0 failures.
  - `cargo clippy --all-targets --all-features -- -D warnings`: 0 warnings.
  - `cargo fmt --check`: 0 issues.
  - `cd apps/vscode-extension && bun test`: 68 passing tests across 11 files (including `real_export_import_e2e.test.ts` and `export_import_preview.test.ts`).
  - `bun run test:integration`: Real VS Code Extension Development Host verified all 26 registered commands, reference opening, and export/import triggers with exit code 0.
  - `bun run package`: Packaged clean `devcontext-0.1.0.vsix` with 0 errors.
- **Documentation:**
  - Specification created at `docs/architecture/export-and-data-safety.md`.

## Phase 8 --- Quality and release readiness

-   [x] Test Windows, Linux, and macOS.
-   [/] Test supported VS Code versions (Partially verified: compile-time @types/vscode ^1.85.0 and current VS Code runtime verified; historical 1.85 binary download deferred).
-   [x] Test spaces and non-ASCII paths.
-   [x] Test read-only workspaces.
-   [/] Test network drives where supported (Partially verified: local UNC share loopback verified; remote physical SMB server unavailable in local environment).
-   [x] Test large repositories (Verified: 2,000-commit repository benchmarked).
-   [x] Test no-Git and multi-root workspaces.
-   [x] Test database lock and recovery.
-   [/] Test upgrade paths (Partially verified: schema migrations v1->v2 verified; full app upgrade is N/A prior to initial release).
-   [x] Audit logs for sensitive data.
-   [x] Audit file and process operations.
-   [/] Review keyboard navigation and accessibility (Partially verified: source-level TreeItem accessibilityInformation and theme tokens verified; external auditory screen reader deferred).
-   [x] Review theme compatibility and contrast.
-   [x] Measure startup and command latency.
-   [x] Write troubleshooting guide.
-   [x] Prepare changelog and release notes.
-   [x] Prepare Marketplace listing and screenshots.
-   [/] Complete release-candidate feedback cycle (Partially verified: internal clean-profile install and workflows verified; external public feedback deferred to Phase 9).

**Verification evidence:**
- **Cross-Platform Compatibility:**
  - Automated CI matrix verified across Windows (`windows-latest`), Linux (`ubuntu-latest`), and macOS (`macos-latest` ARM64) on GitHub Actions (Run ID `41129383061`, all 6 jobs succeeded).
  - Manual and integration verification performed on Windows 11 Pro 64-bit with PowerShell 7.6.6 and Bun 1.4.4.
  - Forward-slash path handling, case preservation, and UNC path validation verified across all file and reference utilities.
- **Network Drives & Edge Cases (`edge_cases_and_workspaces_test.rs` & UNC tests):**
  - Network share workspace: Initialized and executed note CRUD, status, and search on local Windows UNC path (`\\localhost\C$\...`). Inaccessible network paths (`\\nonexistent_host_xyz\...`) return structured `IO_ERROR` (OS error 53) without hanging or panicking.
  - Large Git repositories: Tested repository with 2,000 commits generated via `git fast-import`. `devcontext timeline --limit 50` returned in 131.34 ms; unbounded timeline retrieved within 134.76 ms respecting the 1,000-commit safety cap (`DEFAULT_MAX_COMMITS_UNBOUNDED`) with `is_truncated: true`.
  - Path with spaces (`test_workspace_with_spaces_in_path`): Project initialization, note/decision CRUD, search indexing, and export succeed in paths containing multiple space characters.
  - Unicode and non-ASCII paths (`test_workspace_with_unicode_and_turkish_path`): Full functionality verified in Turkish/Unicode folder names (`proje-geliştirme-dizini/proje-klasörü`).
  - Non-Git repositories (`test_non_git_workspace_lifecycle`): DevContext operates seamlessly in standalone folders without Git; timeline and git metadata degrade cleanly without errors.
  - Multi-root workspace isolation (`test_multi_root_workspace_strict_isolation`): Multiple independent projects maintain separate SQLite databases, schemas, records, and search indexes without data bleeding or cross-contamination.
  - Database locking: Concurrency verified under WAL mode (readers not blocked by active uncommitted writers). Exclusive lock timeouts safely trigger `DATABASE_ERROR: database is locked` after busy-timeout without crashing.
  - Corruption handling (`test_database_corruption_detection_and_safe_reporting`): Corrupted database headers or invalid SQLite files trigger actionable errors without process panics.
  - Schema migration idempotence (`test_schema_migration_idempotence_and_preservation`): Repeated migration cycles on existing databases preserve existing records, schema integrity, and search indexes.
- **Scale and Search Benchmark (`performance_and_scale_test.rs` & startup benchmarks):**
  - Search latency on 10,000 records (SQLite FTS5, BM25 ranking, snippet extraction across 10 queries): Min = 81.26ms, Avg = 102.13ms, Max = 134.85ms (Target: <300ms, achieved ~3x faster).
  - CLI warm startup latency (`devcontext --help`, 10 measured runs after 2 warm-ups): Min = 10.60ms, Avg = 11.57ms, Max = 12.55ms (Target: <200ms, achieved ~17x faster).
  - CLI warm status latency (`devcontext status --json`, 10 measured runs): Min = 14.58ms, Avg = 15.41ms, Max = 16.47ms.
  - CLI cold startup simulation (fresh executable copy to bypass path cache): 340.11ms (including OS Defender/SmartScreen initial validation).
- **Accessibility & Theme Tokens:**
  - Added explicit `accessibilityInformation` (`label` and `role: 'treeitem'`) across all VS Code tree items (`NoteItem`, `DecisionItem`, `TimelineTreeItem`, and `OverviewItem`) in `apps/vscode-extension/src/views/`.
  - All UI elements use semantic VS Code theme tokens (`charts.blue`, `testing.iconPassed`, `problemsWarningIcon.foreground`) adhering to high-contrast and dark/light theme standards.
- **Security & Privacy Audit:**
  - Zero telemetry or network calls verified across core and extension.
  - Zero arbitrary command execution: `saved_commands` are passive text records only; no subprocess execution.
  - Child processes spawned with `shell: false` and strict timeouts.
  - Path redaction and Zip Slip protection verified during export and import operations.
- **Release Candidate Artifacts & Workflow Verification:**
  - VSIX package (`apps/vscode-extension/devcontext-0.1.0.vsix`) test-installed cleanly into isolated VS Code extensions directory (`code --install-extension --force`) and verified via `code --list-extensions`.
  - Standalone release binary built: `target/release/devcontext.exe` (4.05 MB).
  - Release documentation: `CHANGELOG.md`, `docs/user/`, and `docs/release/` complete.

## Phase 9 --- Public MVP release

-   [/] Publish source repository (Prepared for publication: README, dual licenses, security policy, and contributing guidelines verified; awaiting user approval to make public / push).
-   [/] Publish VS Code extension (Prepared for publication: devcontext-0.1.0.vsix built, verified with clean install; awaiting user approval for Marketplace publish).
-   [/] Publish CLI binaries (Prepared for publication: Windows x64 zip and SHA256 checksums packaged in dist/release/; .github/workflows/release.yml configured; awaiting user approval for GitHub Release).
-   [x] Publish documentation and examples (Completed: docs/user/quickstart-and-workflows.md, installation, troubleshooting, backup, privacy).
-   [x] Add sample project data (Completed: examples/sample-project/aurora-sample-export.json, README, and automated integration test).
-   [x] Add issue templates and support guidelines (Completed: bug_report.md, feature_request.md, documentation.md, and .github/SUPPORT.md).
-   [/] Publish release announcement/demo (Prepared for publication: docs/release/public-announcement.md and docs/release/demo-script.md ready; broadcast pending approval).
-   [/] Triage early feedback (Framework established: docs/release/feedback-and-triage-guide.md; awaiting incoming reports post-release).
-   [/] Prioritize reproducible bugs (Framework established: docs/release/feedback-and-triage-guide.md).

**Verification evidence:**
- **Public Source Repository Readiness:**
  - Audited `README.md`, `LICENSE-MIT`, `LICENSE-APACHE`, `SECURITY.md`, `CONTRIBUTING.md`, and `CODE_OF_CONDUCT.md`. Verified consistent naming, dual MIT/Apache-2.0 licensing, and private vulnerability disclosure instructions.
  - Zero secrets, local databases, or machine-specific credentials present in the working tree.
- **VS Code Extension Release Packaging:**
  - Manifest (`package.json`) verified: `publisher: devcontext`, `name: devcontext`, `version: 0.1.0`, `engines.vscode: ^1.85.0`, `categories: ["Productivity", "Other"]`, and relevant discovery keywords.
  - Built production bundle (`dist/extension.js`, 74.64 KB) and packaged `apps/vscode-extension/devcontext-0.1.0.vsix` (24.02 KB, 8 files total, verified dual MIT/Apache-2.0 license text in `LICENSE.txt`, zero development or test files included).
  - VSIX test-installed into isolated VS Code profile (`code --install-extension devcontext-0.1.0.vsix --force`) and verified via `code --list-extensions`.
- **CLI Release Artifacts & Checksums (`dist/release/`):**
  - Compiled release binary: `target/release/devcontext.exe` (4.05 MB).
  - Smoke tests verified: `--help`, `--version`, `doctor`, `init`, note CRUD, decision CRUD, FTS5 search, timeline, and export/import.
  - Packaged Windows release archive: `devcontext-v0.1.0-windows-x64.zip` (1.90 MB, containing `devcontext.exe`, `README.md`, `LICENSE-MIT`, `LICENSE-APACHE`).
  - Multi-platform GitHub Actions release workflow created at `.github/workflows/release.yml` covering Windows x64, Linux x64, macOS Intel x64, and macOS Apple Silicon aarch64.
  - SHA-256 Checksums (`dist/release/SHA256SUMS.txt`):
    - `01f9ff8ac8ccdd0cc99f23b25da88055e0c6cf0805aeab75a63b72724786d7d1`  `devcontext-v0.1.0-windows-x64.zip`
    - `fa0175c658936e51890950d5ca679cd20e92b17a773ab35cca0586fa09191e88`  `devcontext-memory-0.1.0.vsix`
    - `8fc0a98302559a317bd19baddea8c98b897b575eb95ceb7371464e0e8fdcc487`  `devcontext.exe`
- **Documentation and Workflow Examples:**
  - Created comprehensive user guide: `docs/user/quickstart-and-workflows.md` covering end-to-end CLI and VS Code workflows, references, search, timeline, export/import, and disaster recovery.
- **Sample Project Dataset (`examples/sample-project/`):**
  - Created `aurora-sample-export.json` (7 records: 1 note, 2 ADRs, 2 milestones, 1 log entry, 1 saved command) adhering to `schemas/export-v1.schema.json`.
  - Created guide `examples/sample-project/README.md` with inspect, import, search, and cleanup instructions.
  - Automated integration test `sample_project_test.rs` validates parsing, inspection, importing, FTS5 searching, and safe deletion.
- **Issue Templates & Community Guidelines:**
  - Created `.github/SUPPORT.md` outlining triage guidelines, privacy warnings, and response expectations.
  - Refined `.github/ISSUE_TEMPLATE/bug_report.md` with explicit warnings against pasting credentials, tokens, or raw databases, and redirection to `SECURITY.md`.
  - Refined `.github/ISSUE_TEMPLATE/feature_request.md` with local-first and privacy assessment.
  - Created `.github/ISSUE_TEMPLATE/documentation.md`.
- **Public Announcement and Demo Script:**
  - Created `docs/release/public-announcement.md` (concise release announcement draft).
  - Created `docs/release/demo-script.md` (privacy-safe 6-scene demo script and screenshot requirements).
  - Created `docs/release/feedback-and-triage-guide.md` (zero-telemetry feedback taxonomy and triage workflow).
- **Test Suites Clean Pass:**
  - Rust workspace: 110 tests passed, 0 failed (`cargo test --workspace`).
  - VS Code extension: 68 tests passed, 0 failed (`cd apps/vscode-extension && bun test`).
  - TypeScript type-check: 0 errors (`bun run typecheck`).
  - Formatting & lints: `cargo fmt --check` and `cargo clippy --all-targets --all-features -- -D warnings` 0 warnings.

------------------------------------------------------------------------

## Completed work log

Add newest entries first.

  Date         Phase / task                               Change                                                                Verification evidence
  ------------ ------------------------------------------ --------------------------------------------------------------------- ---------------------------------------------------------------------------------
  2026-10-31   Phase 9: Version Bump to v0.1.1            Bumped version to 0.1.1 across Cargo workspace, core version assert, and VS Code package.json. Packaged `devcontext-memory-0.1.1.vsix` (`9d4aaea0babedb29602ed8ea3e033c6ded4c79efc0e684ca3d2f461315c579c0`). 110 Rust tests + 68 TS tests passing.
  2026-10-31   Phase 9: Crate-specific README Redesign    Created comprehensive standalone READMEs for `devcontext-core` (code examples, module map, docs.rs badges) and `devcontext-cli` (commands, quickstart, cargo install). Eliminated broken relative links. Verified `cargo package`.
  2026-10-31   Phase 9: Crates.io Public Release          Published `devcontext-core` v0.1.0 and `devcontext-cli` v0.1.0 to crates.io. Live packages verified via `cargo search devcontext`. Updated installation guides with `cargo install devcontext-cli`.
  2026-10-31   Phase 9: Activity Bar Monochrome Icon      Redesigned `media/icon.svg` as a dedicated 24x24 transparent monochrome SVG (`currentColor`) for VS Code Activity Bar, eliminating solid white squircle mask artifact. Preserved full branding vector in `media/logo.svg` and Marketplace PNG in `media/icon.png`. Repackaged VSIX (`f0b11d5a352d2d61b497e279d11e9891ce729464dabe1fa0f98010bb1f8b9983`). 68 TS tests passing.
  2026-10-31   Phase 9: Extension Icon Design & Bundling  Created custom modern vector SVG icon (`media/icon.svg`) and high-res 256x256 PNG (`media/icon.png`) for VS Code Marketplace. Configured `"icon": "media/icon.png"` in package.json. Verified VSIX packaging and `<Icon>` manifest entry.
  2026-10-31   Phase 9: Marketplace Category Validation   Resolved VS Code Marketplace upload rejection ("category 'Productivity' not available in 'en-us'"): replaced invalid "Productivity" category with official "Other" and added "productivity" to keywords. Packaged `devcontext-memory-0.1.0.vsix`.
  2026-10-31   Phase 9: Marketplace DisplayName Collision Updated VS Code extension Marketplace displayName to "DevContext by Taiizor" (extension ID `Taiizor.devcontext-memory`) to resolve global displayName collision. Preserved DevContext product branding in UI/commands/views/docs. Packaged `devcontext-memory-0.1.0.vsix`. All tests passing.
  2026-10-31   Phase 9: Marketplace Identity Migration    Updated VS Code extension Marketplace package name to `devcontext-memory` (extension ID `Taiizor.devcontext-memory`) to resolve global Marketplace name collision. Preserved DevContext product name, `devcontext.*` command IDs, and storage. Packaged `devcontext-memory-0.1.0.vsix`. All tests passing.
  2026-10-31   Phase 9: Publisher Identity & Ownership    Migrated repository references to https://github.com/Taiizor/DevContext, updated VS Code publisher to Taiizor (extension ID Taiizor.devcontext-memory), updated integration test fixtures and installation/migration docs, preserved product identifiers and internal namespaces. All tests and VSIX packaging verified.
  2026-10-31   Phase 9: Release Gate & Git Reconciliation Reconciled Git history (HEAD at beb792f/commit 25, verified ancestor 65028a3/23 via a4334bc/24), verified working tree, corrected release tag filter to exact stable pattern `v[0-9]+.[0-9]+.[0-9]+` in release.yml. Git history clean and read-only verified.
  2026-10-31   Phase 9: Public MVP Release Preparation    Packaged CLI release archive and SHA256 checksums, prepared production VSIX, added sample project dataset (Aurora Engine) with automated integration test, created Quickstart guide, support guidelines (.github/SUPPORT.md), issue templates, announcement draft, demo script, and triage guide. 110 Rust tests + 68 TS tests passing. Publication approval gates documented.
  2026-10-31   Phase 8: Quality & Release Readiness       Conducted cross-platform CI matrix verification (Win/Mac/Linux), added edge case tests (spaces, non-ASCII/Turkish paths, non-git, multi-root isolation, corruption), 10k-record FTS5 search benchmark, CLI cold-start latency measurements, accessibility treeitem metadata across VS Code views, release docs, user guides, changelog, and VSIX/binary builds. 109 Rust tests + 68 TS tests (Bun) passing. GitHub Actions CI run 41129383061 (all 6 jobs succeeded).
  2026-10-30   Phase 7: Export/Import & Data Safety       Implemented versioned JSON (export-v1.json), human-readable Markdown hierarchy, portable ZIP archives, path sanitization/redaction, conflict strategies, dry-run, pre-import/pre-delete automatic backups, data inspection, complete deletion, and VS Code commands. 84 Rust tests + 68 TS tests (Bun) + Real VS Code Extension Host (exit code 0) passing.
  2026-10-30   Phase 6: VS Code Extension MVP             Implemented VS Code Extension MVP (ADR 0012, manifest, CoreLocator, CoreClient IPC, TreeViews, QuickPick search, reference actions including real Git commit diffs, status bar, and test suite). Real Extension Host verification + 62 TS tests (Bun) + 72 Rust tests passing. VSIX package clean.
  2026-10-30   Phase 5: Search & References               Implemented SQLite FTS5 search (ADR 0011), Turkish search normalization, BM25 ranking, ReferenceResolver with canonical traversal security, CLI search command, and test suite. 72 Rust tests + 2 TS tests passing.
  2026-10-30   Phase 4: Git Integration & Timeline        Implemented read-only GitAdapter (ADR 0010), TimelineService, CLI timeline command, and integration tests. 48 tests passing.
  2026-10-30   Phase 3: Independent CLI MVP               Implemented CLI subcommands (init, status, doctor, note, decision, milestone, log), JSON protocol, auto-discovery, and integration test suite. 41 tests passing.
  2026-10-30   Phase 2: Core Domain & Storage Engine      Implemented domain models, SQLite WAL engine, schema v1 migrations, backup/restore primitives, and 30 unit/integration tests. `cargo fmt`, `cargo clippy`, and `cargo test` passed with 0 warnings.
  2026-10-30   Phase 1: Repository & Foundation           Established Cargo & Bun workspaces, licenses, CI, docs, test suites. `cargo fmt`, `cargo clippy`, `cargo test`, `tsc --noEmit`, `bun test`, `bun run build` all passed cleanly.
  2026-10-30   Phase 0: Discovery & Product Contract      Created PRD, Architecture Overview, MVP Checklist, JSON Schema v1, and ADRs 0003-0009. Updated PROGRESS.md. Validated schema syntax with Bun. Checked git status.

## Deferred decisions

  -----------------------------------------------------------------------
  Topic             Decision needed   Deadline /        Status
                                      trigger           
  ----------------- ----------------- ----------------- -----------------
  Global storage    Decide whether to Before storage    Deferred to
                    support a global  implementation    post-MVP; default
                    data directory in (Phase 2)         is .devcontext/
                    MVP.                                

  Core distribution Decide packaging  Before extension  Resolved in ADR
                    strategy for Rust MVP (Phase 6)     0012: Platform-
                    core binaries                       specific VSIX
                    used by the                         packages with
                    extension.                          layered fallback.
  -----------------------------------------------------------------------

## Update protocol for agents

After every meaningful work session: 1. Update the current phase and
overall status. 2. Mark only verified tasks as complete. 3. Add exact
test commands and outcomes to verification evidence. 4. Record changed
files or relevant commit hashes in the completed work log. 5. Add
blockers with a concrete next action. 6. Update the phase summary table.
7. If scope or architecture changed, update `ROADMAP.md` and
create/update an ADR. 8. Never mark a task complete merely because code
was written; run the relevant checks.
