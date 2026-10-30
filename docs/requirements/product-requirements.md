# DevContext --- Product Requirements Document (PRD)

- **Product Name:** DevContext
- **Status:** Approved for Phase 0 / MVP Planning
- **Version:** 1.0.0
- **Target Audience:** Individual software developers, open-source maintainers, and small teams seeking a reliable, local-first repository memory system.

---

## 1. Executive Summary & Vision

Software projects evolve over months and years, but the human rationale behind critical engineering decisions is frequently lost. Git records *how* lines of code changed, but rarely preserves the full narrative of *why* choices were made, what alternatives were rejected, or how to resume work after a long absence.

DevContext is a local-first, privacy-by-default developer memory system. It preserves authored technical decisions, contextual notes, milestones, references, reusable commands, and timeline records directly on the developer's local machine, operating without mandatory accounts, AI services, network access, or cloud infrastructure.

---

## 2. Product Identity & Platform Support

- **Product Name:** DevContext
- **Repository Name:** `DevContext`
- **Recommended License:** Dual MIT / Apache-2.0 (permissive open-source licensing aligned with Rust and TypeScript developer ecosystems).
- **Supported Operating Systems:**
  - Windows 11 / Windows 10 (x64, ARM64)
  - Linux (x64, ARM64 - Ubuntu 20.04+, Debian 11+, Fedora, Arch)
  - macOS (Apple Silicon ARM64, Intel x64 - macOS 12+)
- **Client Targets:**
  - DevContext CLI: Standalone native binary.
  - DevContext VS Code Extension: Native editor integration via typed protocol to core binary.
- **Runtimes & Toolchains:**
  - Rust 1.80+ (Core domain, persistence, CLI engine).
  - TypeScript 5.5+ and Bun runtime for extension development and packaging.
  - SQLite 3 (bundled / statically linked with FTS5 enabled).

---

## 3. Scope Definition

### 3.1 In-Scope for MVP (Phases 0–9)

1. **Workspace & Identity:**
   - Git repository detection and non-Git directory support.
   - Stable UUID-based project identity independent of absolute filesystem paths.
   - Multi-root workspace isolation (no cross-contamination of project databases).
2. **Knowledge Records:**
   - Developer notes with Markdown formatting, tags, categories, and pinning.
   - Architecture Decision Records (ADRs) with formal lifecycle states (`proposed`, `accepted`, `rejected`, `deprecated`, `superseded`).
   - Milestones and manual development log entries.
   - Saved commands (manual opt-in only, with secret warnings and explicit user confirmation before running).
3. **References:**
   - Typed references to local files, folders, code symbols, Git commits, branches, URLs, and other DevContext records.
   - Graceful degradation when external targets move or are deleted.
4. **Search & Timeline:**
   - Fast, offline full-text search across titles, markdown bodies, and tags using SQLite FTS5.
   - Unified chronological timeline combining authored DevContext records with read-only Git commit metadata.
5. **Storage, Portability & Privacy:**
   - Local-first persistence in `.devcontext/devcontext.db`.
   - Local git exclusion via `.git/info/exclude` by default.
   - Export to human-readable Markdown directory, versioned JSON (v1), and combined ZIP archive.
   - Safe import with schema validation, preview, ID collision handling, and pre-import backups.
   - Hard deletion workflow and data inspection utilities (`doctor`).

### 3.2 Explicitly Out of Scope for MVP

The following items are deferred until post-MVP validation:
- **No AI / LLM dependencies:** No automatic summary generation, automated decision inference, or autonomous code analysis.
- **No Cloud / Accounts:** No hosted sync servers, remote accounts, telemetry, or analytics collection.
- **No Background Surveillance:** No automatic sniffing of terminal keystrokes, environment variables, or silent monitoring of arbitrary file edits.
- **No Source Code Modification:** DevContext does not rewrite user source code, build scripts, or Git commit history.
- **No Issue Tracker Replacement:** DevContext does not seek to replace Jira, Linear, or GitHub Issues.

---

## 4. User Stories & Acceptance Criteria

### US-1: Project Initialization & Identity
- **As a** developer opening a project repository,
- **I want to** initialize DevContext with a single command,
- **So that** my notes and decisions are anchored to a stable project identity without cluttering public Git history.
- **Acceptance Criteria:**
  - `devcontext init` creates `.devcontext/` directory and `.devcontext/devcontext.db`.
  - Assigns a immutable UUIDv4 project identifier.
  - Automatically recommends adding `.devcontext/` to `.git/info/exclude` to ensure local privacy by default without altering tracked `.gitignore`.
  - Works identically on Git repositories and standalone directories.
  - If already initialized, reports existing status and does not overwrite existing data.

### US-2: Developer Notes Management
- **As a** developer working on a complex feature,
- **I want to** write contextual markdown notes linked to tags and categories,
- **So that** I can capture thoughts and context as I work.
- **Acceptance Criteria:**
  - Notes accept `title`, `body` (Markdown), optional `tags`, and optional `category`.
  - Notes store exact UTC timestamps for creation and modification.
  - Notes can be pinned, archived, updated, and deleted.
  - Note list supports filtering by tag and archive state.

### US-3: Architecture Decision Records (ADRs)
- **As a** technical lead or solo developer,
- **I want to** record architecture decisions following an ADR structure,
- **So that** future maintainers understand the rationale and trade-offs.
- **Acceptance Criteria:**
  - Decisions support structured fields: `title`, `status`, `context`, `decision`, `consequences`, optional `alternatives_considered`.
  - Status transitions: `proposed` -> `accepted` | `rejected` -> `deprecated` | `superseded`.
  - When superseded, the record explicitly links to the replacing decision ID; historical records are never overwritten.
  - Decisions can be exported as standalone ADR markdown files.

### US-4: Milestones & Development Log
- **As a** project maintainer,
- **I want to** record milestones and notable events in a development log,
- **So that** I have a high-level changelog of conceptual progress.
- **Acceptance Criteria:**
  - Milestones store title, description, status (`planned`, `active`, `completed`, `cancelled`), and target/completion dates.
  - Log entries record event date, summary, and details.
  - Both milestone and log records surface directly on the chronological timeline.

### US-5: Typed References
- **As a** developer reviewing a decision or note,
- **I want to** link specific files, symbols, Git commits, or URLs to the record,
- **So that** I can jump directly to relevant context.
- **Acceptance Criteria:**
  - Supported reference types: `file`, `folder`, `symbol`, `git_commit`, `git_branch`, `url`, `command`, `record`.
  - File references use workspace-relative paths to ensure portability across clones.
  - If a referenced file or commit is missing, the UI/CLI marks the reference as "Unavailable" without corrupting or deleting the parent record.

### US-6: Full-Text Search & Filtering
- **As a** developer returning to a project after months,
- **I want to** search across all past notes, decisions, and milestones by keyword,
- **So that** I can rapidly rediscover how a specific problem was solved.
- **Acceptance Criteria:**
  - Full-text search operates offline using SQLite FTS5 over titles, Markdown bodies, and tags.
  - Search returns relevant snippets with query highlights.
  - Supports filters by record type, status, tag, and date range.
  - Search query response time < 300 ms for datasets up to 10,000 records.

### US-7: Project Timeline
- **As a** developer catching up on project progress,
- **I want to** see an integrated timeline of authored decisions, notes, and Git commit metadata,
- **So that** I have a holistic view of technical evolution.
- **Acceptance Criteria:**
  - Timeline displays DevContext records and Git commits chronologically.
  - Authored records and Git commits are visually and structurally distinguished.
  - Git integration is strictly read-only and respects configurable commit depth limit.
  - If Git is not installed or repo has no commits, timeline displays DevContext records gracefully without crashing.

### US-8: Opt-In Saved Commands
- **As a** developer working with complex build/test scripts,
- **I want to** save reusable commands with explanations and working directories,
- **So that** I can recall how to run tests or migrations without risking accidental execution.
- **Acceptance Criteria:**
  - Commands are saved only upon explicit user action.
  - Heuristic check warns users if potential secrets/tokens appear in command text.
  - Saved commands are NEVER executed automatically. Execution requires explicit confirmation in the UI/CLI.

### US-9: Export & Import
- **As a** developer migrating to a new laptop or sharing project context,
- **I want to** export all project knowledge into standard formats and import them back,
- **So that** my data is completely portable and vendor-neutral.
- **Acceptance Criteria:**
  - Export supports:
    1. Human-readable Markdown folder structure.
    2. Versioned JSON bundle (`export-v1.json`).
    3. Portable ZIP archive combining both.
  - Export excludes absolute machine paths by default.
  - Import validates schema version, previews additions/modifications, detects UUID collisions, and creates a pre-import backup.

### US-10: Privacy, Security & Data Safety
- **As a** security-conscious developer,
- **I want to** be certain my private code, secrets, and environment variables are protected,
- **So that** DevContext remains a trustworthy local utility.
- **Acceptance Criteria:**
  - Zero network calls are made by core or extension.
  - Zero telemetry or analytics collected.
  - No automatic reading of `.env` files or process environment variables.
  - Full project deletion command (`devcontext destroy` / `devcontext purge`) cleanly removes `.devcontext/` upon double confirmation.

---

## 5. Non-Functional Requirements & Performance Targets

1. **CLI Startup Latency:** Common CLI commands (e.g., `devcontext status`, `devcontext note list`) execute in under 150 ms.
2. **Database Integrity:** SQLite with WAL mode (`Write-Ahead Logging`), immediate foreign key constraints, and transactional migrations.
3. **Robustness:** Gracefully handles non-Git workspaces, detached HEAD, spaces in paths, unicode characters, and read-only filesystem detection.
4. **Resource Footprint:** Core executable size kept compact, low memory usage (< 50MB idle), no background daemons running when idle.
