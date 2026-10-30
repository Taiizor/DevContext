# DevContext --- End-to-End Product Roadmap

> **Purpose:** Build a local-first project memory system that preserves
> the development context behind a software repository: decisions,
> notes, milestones, relevant changes, commands, and references.
>
> **Status:** Planning\
> **Roadmap version:** 1.0\
> **Last updated:** 2026-10-30

## 1. Product vision

DevContext is a project memory and development-context tool. Git stores
code history; DevContext stores the human and technical context that
helps developers understand why the project evolved the way it did.

When a developer returns to a repository after weeks or months,
DevContext should help answer:

-   What was I working on?
-   Why was a technical decision made?
-   Which alternatives were considered?
-   What remains unfinished?
-   How do I build, test, and run this project?
-   Which files, symbols, commits, or modules relate to a note or
    decision?
-   What important events happened since my last session?

DevContext must be useful without AI, without an account, without a
network connection, and without a hosted service.

### Product principles

1.  **Local-first and private by default.** Project data stays on the
    user's machine unless the user explicitly exports or shares it.
2.  **Human-owned knowledge.** Important decisions and explanations must
    be explicitly authored or confirmed by the developer. Never silently
    invent project rationale.
3.  **Useful automation, not surveillance.** Automatically collect only
    low-risk, explainable metadata. Do not capture arbitrary terminal
    input, file contents, secrets, or user activity by default.
4.  **Portable data.** Notes and decisions must be exportable in
    documented, stable formats.
5.  **Editor-independent core.** The VS Code extension is a client of a
    reusable core and CLI, not the owner of business logic.
6.  **Language-agnostic.** The core must not depend on one programming
    language or build system.
7.  **Trust and transparency.** Show what is collected, where it is
    stored, and how to delete or export it.
8.  **Incremental delivery.** Ship a small, dependable MVP before adding
    automation and advanced integrations.

## 2. Product scope

### In scope

-   Repository/workspace initialization and identity.
-   Project overview and local project metadata.
-   Developer-authored notes.
-   Architecture Decision Records (ADRs) and technical decision records.
-   Milestones and development log entries.
-   References to files, folders, symbols, URLs, commits, branches, and
    commands.
-   Search across stored project knowledge.
-   A project timeline combining DevContext records with selected Git
    metadata.
-   A VS Code extension.
-   A standalone CLI.
-   Local database, backup, import, and export.
-   Extensible collectors and adapters.
-   Privacy controls and data lifecycle management.

### Explicitly out of scope for the initial release

-   AI-generated explanations or autonomous decision creation.
-   Cloud synchronization, accounts, teams, or hosted dashboards.
-   Full project management / issue tracking.
-   Replacing Git, GitHub, Jira, Linear, or documentation platforms.
-   Automatic recording of all terminal commands.
-   Full semantic code understanding across every language.
-   Background monitoring of arbitrary file activity.
-   Automatic modification of source code.
-   Automatic dependency upgrades or environment changes.

These may be reconsidered only after the core product proves useful.

## 3. Target users and primary workflows

### Target users

-   Individual developers maintaining personal projects.
-   Open-source maintainers.
-   Developers joining an unfamiliar repository.
-   Developers returning to a project after a long break.
-   Small teams that want portable project knowledge without adopting a
    hosted service.

### Primary workflows

**A. Start a project** 1. Open a repository in VS Code. 2. Run
`DevContext: Initialize Project`. 3. DevContext detects the repository
root and asks the user to confirm the storage location. 4. The project
receives a stable DevContext identity. 5. The extension displays an
empty project overview and suggested next actions.

**B. Record a decision** 1. Run `DevContext: New Decision`. 2. Enter
title, status, context, decision, consequences, and optional
alternatives. 3. Link relevant files, commits, or URLs. 4. Save as a
Markdown ADR with metadata. 5. Show it in the timeline and search
results.

**C. Record a note** 1. Select text or a file/symbol, or open the note
command. 2. Enter a title and note body. 3. Attach the current
file/symbol reference if desired. 4. Save locally and make it
searchable.

**D. Return to a project** 1. Open the repository. 2. View recent
DevContext activity and Git activity. 3. Search for a past decision,
command, note, or topic. 4. Open the associated files or commits. 5.
Continue work with less rediscovery.

**E. Export or move knowledge** 1. Run export. 2. Choose Markdown
folder, JSON bundle, or portable archive. 3. Review included data. 4.
Export without including secrets or machine-specific private data by
default. 5. Import into another machine or repository clone.

## 4. Product requirements

### 4.1 Project identity and workspace discovery

-   Identify the Git repository root when available.
-   Support non-Git folders as projects.
-   Support multi-root VS Code workspaces without silently merging
    unrelated projects.
-   Store a stable project identifier independent of the absolute local
    path.
-   Detect when a folder is a different clone of an existing project.
-   Allow the user to link or keep project identities separate.
-   Handle renamed folders and moved repositories.
-   Provide an explicit reset/unlink operation.
-   Never initialize a project without clear user intent.

### 4.2 Project overview

Display: - Project name and root. - Project identifier. - Repository
status and current branch when available. - Recent DevContext records. -
Recent Git commits (metadata only). - Open milestones or pinned notes. -
Quick actions: new note, new decision, add milestone, search, export.

Do not present inferred architecture or language percentages as
authoritative facts in the MVP.

### 4.3 Notes

A note should support: - Title. - Markdown body. - Created and updated
timestamps. - Tags. - Optional category. - Optional links to files,
folders, symbols, commits, branches, URLs, or commands. - Pin/archive
state. - Search indexing.

Notes must be editable and deletable. Preserve predictable behavior when
a referenced file is renamed or deleted.

### 4.4 Decision records

Use an ADR-compatible structure. A decision should support: - Title. -
Status: proposed, accepted, rejected, deprecated, superseded. -
Context/problem. - Decision. - Consequences. - Alternatives considered
(optional). - Related decisions. - References. - Created/updated
timestamps. - Optional decision date and authorship label.

When a decision is superseded, preserve the old record and link it to
the replacement. Do not overwrite historical decisions.

### 4.5 Milestones and development log

Milestones represent meaningful project checkpoints, not a full task
tracker.

Support: - Title and description. - Status: planned, active, completed,
cancelled. - Target date (optional). - Completion date (optional). -
Related notes and decisions. - Manual log entries.

A development log entry may describe a meaningful event, such as a
release, migration, architecture change, or completed investigation.

### 4.6 References

A reference is a typed link from a DevContext record to an external or
repository object.

Initial reference types: - File path. - Folder path. - Symbol
(language-specific details optional). - Git commit. - Git branch. -
URL. - Shell command (stored as text, never executed automatically). -
Milestone. - Another DevContext record.

References must degrade gracefully if their target no longer exists.
Provide a clear "reference unavailable" state rather than deleting the
knowledge record.

### 4.7 Search

MVP search: - Full-text search over titles, bodies, tags, and selected
metadata. - Filters by record type, tag, status, and date. - Results
ordered by relevance and/or recency. - Search snippets. - Keyboard-first
interaction. - Search must work offline.

Use SQLite FTS5 if supported by the chosen SQLite distribution. Keep a
fallback strategy documented if FTS5 is unavailable.

### 4.8 Timeline

Combine: - DevContext notes. - Decisions. - Milestones. - Manual log
entries. - Selected Git commit metadata.

Clearly distinguish authored DevContext records from imported Git
metadata. Git commit messages are not proof of intent; never convert
them into asserted rationale.

Filters: - Record type. - Date range. - Tag. - Git-only /
DevContext-only / combined.

### 4.9 Command history

For the MVP, command history is **manual and opt-in**: - User can save a
command with a title, explanation, tags, and optional working
directory. - Commands are stored as text. - A command is never executed
automatically. - Before running a saved command, show the exact command
and require confirmation. - Do not capture terminal input globally. -
Redact or warn about likely secrets when saving commands. - Support
shell-specific notes, but do not assume shell portability.

Automatic terminal integration is a later, separately consented feature.

### 4.10 Export and import

Required formats: - Markdown directory export, readable without
DevContext. - JSON export with a versioned schema. - Portable archive
containing the JSON bundle and Markdown files.

Import requirements: - Validate schema version. - Preview changes before
applying. - Detect duplicate IDs. - Offer merge, skip, or replace
policies. - Never overwrite existing data without explicit
confirmation. - Preserve record IDs and timestamps. - Produce an import
report.

### 4.11 Privacy and security

-   Local storage by default.
-   No telemetry in the initial release.
-   No network requests required for core workflows.
-   Never collect environment variable values automatically.
-   Never collect secrets, credentials, arbitrary file contents, or
    terminal input automatically.
-   Exports must exclude machine-specific absolute paths by default or
    clearly offer a path-redaction option.
-   Add a data inspection command and a complete project-data deletion
    flow.
-   Use parameterized SQL queries.
-   Validate imported data and cap unreasonable payload sizes.
-   Treat repository contents and imported bundles as untrusted input.
-   Do not execute content from notes, commands, or imports.

## 5. Recommended architecture

### 5.1 High-level structure

``` text
devcontext/
├── apps/
│   ├── vscode-extension/       # VS Code UI and commands
│   └── cli/                     # Standalone CLI
├── crates/
│   ├── devcontext-core/         # Domain logic and use cases
│   ├── devcontext-storage/      # SQLite persistence and migrations
│   ├── devcontext-search/       # Full-text search
│   ├── devcontext-git/          # Git metadata adapter
│   ├── devcontext-export/       # Markdown, JSON, archive
│   └── devcontext-protocol/     # Stable CLI/extension interface
├── packages/
│   └── vscode-client/           # Optional shared TypeScript client
├── schemas/
│   ├── export-v1.schema.json
│   └── config-v1.schema.json
├── docs/
│   ├── architecture/
│   ├── decisions/
│   ├── security/
│   └── user-guide/
├── fixtures/
└── tests/
```

This is a suggested logical layout, not a requirement to create empty
crates immediately. Avoid premature fragmentation. Start with a small
workspace and split modules only when boundaries are clear.

### 5.2 Technology recommendation

**Core and CLI:** Rust\
**VS Code extension:** TypeScript\
**Database:** SQLite\
**Search:** SQLite FTS5\
**Inter-process communication:** JSON-RPC over stdio or a versioned CLI
protocol\
**Export:** Markdown + versioned JSON + ZIP-compatible archive\
**Testing:** Rust unit/integration tests; TypeScript extension tests;
end-to-end tests on supported operating systems.

Rationale: - Rust provides a portable native core and CLI. - TypeScript
is the natural choice for the VS Code extension API. - SQLite is
portable, local, inspectable, and easy to back up. - A versioned
protocol prevents the extension from depending on internal Rust
implementation details.

Do not make the extension shell out to arbitrary shell strings. Invoke a
known executable with structured arguments. Validate paths and
arguments.

### 5.3 Core boundaries

The core owns: - Domain models. - Validation. - Use cases. - Project
identity. - Storage transactions. - Search semantics. - Import/export
semantics. - Timeline assembly.

Adapters own: - Git discovery and metadata. - VS Code workspace
integration. - Filesystem access. - Platform-specific behavior.

The UI must not contain business rules. The CLI and extension must call
the same core use cases.

### 5.4 Storage location

Recommended default: - Project data stored inside a hidden
`.devcontext/` directory in the project root. - Database file:
`.devcontext/devcontext.db`. - Optional attachments:
`.devcontext/attachments/`. - Exported backups should be written to a
user-selected destination.

Provide a setting for a global data directory for users who do not want
project metadata inside the repository.

Do not automatically add `.devcontext/` to `.gitignore`. Ask the user
whether project knowledge should be shared with the repository. Explain
that the database may contain private notes. Offer a safe default and a
clear choice: - Keep local and add `.devcontext/` to
`.git/info/exclude`. - Share selected Markdown knowledge through Git. -
Manage ignore rules manually.

The recommended default is local-only data, excluded from Git without
modifying the shared `.gitignore`.

### 5.5 Data model (initial proposal)

Core entities: - `Project` - `Record` - `Note` - `Decision` -
`Milestone` - `LogEntry` - `Reference` - `Tag` - `RecordTag` -
`GitEvent` - `SavedCommand` - `SchemaMigration`

Use stable UUIDs for records. Store timestamps in UTC ISO-8601 or
integer Unix milliseconds consistently; document the choice. Keep
human-facing dates localized by the client.

Prefer a shared `Record` envelope for common fields, with typed payload
tables or a validated type-specific JSON payload. Choose one model
during architecture design and document trade-offs. Avoid an
over-generalized EAV schema.

## 6. Delivery roadmap

The phases below are sequential gates. A phase is complete only when its
acceptance criteria pass. Track actual task completion in `PROGRESS.md`,
not by editing this roadmap.

### Phase 0 --- Discovery and product contract

**Goal:** Resolve product decisions before implementation.

Tasks: - \[ \] Confirm product name, repository name, license, and
supported platforms. - \[ \] Define MVP and explicitly reject non-MVP
features. - \[ \] Write user stories for initialization, notes,
decisions, search, timeline, and export. - \[ \] Define privacy policy
and collection boundaries. - \[ \] Decide default storage location and
Git ignore behavior. - \[ \] Decide multi-root workspace behavior. - \[
\] Define stable project identity and clone detection behavior. - \[ \]
Choose Rust/TypeScript workspace strategy. - \[ \] Define CLI-to-core
and extension-to-core communication. - \[ \] Draft initial data model. -
\[ \] Draft export schema version 1. - \[ \] Record major architecture
decisions in `docs/decisions/`.

Deliverables: - Product requirements document. - Architecture
overview. - Initial ADRs. - MVP acceptance checklist.

Exit criteria: - Every MVP workflow has a written acceptance
criterion. - Data collection and privacy behavior are explicit. - No
unresolved decision blocks implementation.

### Phase 1 --- Repository and engineering foundation

**Goal:** Create a maintainable, testable cross-platform repository.

Tasks: - \[ \] Create repository structure. - \[ \] Add root `AGENTS.md`
and contribution guidance. - \[ \] Add license, README, SECURITY,
CODE_OF_CONDUCT, and CONTRIBUTING documents. - \[ \] Configure
formatting and linting for Rust and TypeScript. - \[ \] Configure CI for
Windows, Linux, and macOS. - \[ \] Add dependency update policy. - \[ \]
Add conventional commit or another documented commit convention. - \[ \]
Add issue and pull request templates. - \[ \] Add build scripts and
developer setup instructions. - \[ \] Add test fixtures and sample
repositories. - \[ \] Add release versioning strategy. - \[ \] Ensure no
secrets or local database files are committed.

Deliverables: - Buildable repository skeleton. - CI pipeline. -
Developer setup guide.

Exit criteria: - Clean checkout builds on all supported CI operating
systems. - Formatting, linting, and baseline tests pass.

### Phase 2 --- Core domain and storage

**Goal:** Implement the reliable local data foundation.

Tasks: - \[ \] Define domain types and validation rules. - \[ \]
Implement project initialization and stable identity. - \[ \] Implement
SQLite connection management. - \[ \] Implement schema migrations. - \[
\] Implement transactional repository interfaces. - \[ \] Implement
create/read/update/delete for notes. - \[ \] Implement
create/read/update/delete for decisions. - \[ \] Implement milestone and
log entry persistence. - \[ \] Implement tags and references. - \[ \]
Implement saved command records (storage only). - \[ \] Implement data
integrity checks. - \[ \] Implement database backup and restore
primitives. - \[ \] Add migration tests from every released schema
version. - \[ \] Add corruption and locked-database handling. - \[ \]
Add tests for concurrent access and interrupted writes.

Deliverables: - Working core library. - Versioned SQLite schema. -
Domain and storage tests.

Exit criteria: - Core CRUD workflows work without VS Code. - Data
survives restart. - Migrations and backup/restore tests pass.

### Phase 3 --- CLI MVP

**Goal:** Make DevContext useful before building the editor UI.

Commands to implement: - \[ \] `devcontext init` - \[ \]
`devcontext status` - \[ \] `devcontext note add` - \[ \]
`devcontext note list` - \[ \] `devcontext note show` - \[ \]
`devcontext note edit` - \[ \] `devcontext note delete` - \[ \]
`devcontext decision add` - \[ \] `devcontext decision list` - \[ \]
`devcontext decision show` - \[ \] `devcontext decision update` - \[ \]
`devcontext milestone add` - \[ \] `devcontext log add` - \[ \]
`devcontext search` - \[ \] `devcontext timeline` - \[ \]
`devcontext export` - \[ \] `devcontext import` - \[ \]
`devcontext doctor`

Additional requirements: - \[ \] Provide `--help` and useful error
messages. - \[ \] Support machine-readable JSON output where
appropriate. - \[ \] Use non-zero exit codes for failures. - \[ \] Never
prompt in non-interactive mode unless required. - \[ \] Provide `--yes`
only for safe, clearly documented operations. - \[ \] Never execute
saved commands as a side effect. - \[ \] Add CLI integration tests.

Deliverables: - Installable CLI for supported platforms. - CLI manual
and examples.

Exit criteria: - A user can initialize a project, add notes and
decisions, search, view a timeline, and export/import without VS Code.

### Phase 4 --- Git integration and timeline

**Goal:** Add useful project history without confusing metadata with
human knowledge.

Tasks: - \[ \] Detect Git repository root. - \[ \] Read current branch
and HEAD. - \[ \] Read commit metadata: hash, author label, date,
subject. - \[ \] Support configurable history depth. - \[ \] Associate
DevContext records with commits manually. - \[ \] Allow a user to link
the current HEAD to a decision or note. - \[ \] Assemble a combined
timeline. - \[ \] Label Git events distinctly from authored records. -
\[ \] Handle shallow clones and missing Git executables gracefully. - \[
\] Handle detached HEAD and repositories without commits. - \[ \] Do not
modify Git history or hooks. - \[ \] Add Git fixture tests.

Deliverables: - Git adapter. - Combined timeline API and CLI view.

Exit criteria: - Git metadata is read-only and clearly distinguished. -
Timeline remains useful when Git is unavailable.

### Phase 5 --- Search and references

**Goal:** Make stored knowledge easy to retrieve.

Tasks: - \[ \] Implement full-text indexing. - \[ \] Index titles,
Markdown bodies, tags, and selected metadata. - \[ \] Implement result
snippets and ranking. - \[ \] Add filters for record type, status, tag,
and date. - \[ \] Implement file/folder references. - \[ \] Implement
commit and branch references. - \[ \] Implement URL references. - \[ \]
Implement command references. - \[ \] Add reference resolution and
unavailable-target states. - \[ \] Add reindex command. - \[ \] Add
search performance tests using a realistically sized fixture. - \[ \]
Document indexing behavior and limitations.

Deliverables: - Search API and CLI. - Typed reference system.

Exit criteria: - Search is offline, deterministic, and fast on the
agreed test dataset. - Missing targets never destroy stored records.

### Phase 6 --- VS Code extension MVP

**Goal:** Deliver a polished, keyboard-first editor experience.

Tasks: - \[ \] Create extension manifest and activation strategy. - \[
\] Detect workspace roots and multi-root workspaces. - \[ \] Locate or
provision the compatible DevContext core executable. - \[ \] Implement
version handshake between extension and core. - \[ \] Add project
initialization command. - \[ \] Add activity bar container and project
overview view. - \[ \] Add notes tree view. - \[ \] Add decisions tree
view. - \[ \] Add timeline view. - \[ \] Add search command and
quick-pick results. - \[ \] Add note creation/editing UI using
Markdown. - \[ \] Add decision creation/editing UI. - \[ \] Add
milestone and log creation commands. - \[ \] Add context actions for
file and selection references. - \[ \] Add open-reference actions for
files and commits. - \[ \] Add export/import commands with preview and
confirmation. - \[ \] Add status bar indicator only if it provides
meaningful state. - \[ \] Add keyboard shortcuts only for high-frequency
actions. - \[ \] Add extension settings for storage, privacy, and
timeline behavior. - \[ \] Add graceful handling when the core binary is
missing or incompatible. - \[ \] Add extension host tests and manual
test checklist. - \[ \] Ensure UI never blocks on long-running core
operations.

Deliverables: - Installable VS Code extension. - Core binary
distribution or documented setup. - User guide and screenshots.

Exit criteria: - All MVP workflows can be completed in VS Code. -
Extension works with a fresh project and an existing initialized
project. - Extension failure cannot corrupt the database.

### Phase 7 --- Export, import, portability, and data safety

**Goal:** Make user knowledge durable and portable.

Tasks: - \[ \] Finalize versioned JSON export schema. - \[ \] Produce
readable Markdown export. - \[ \] Produce portable archive. - \[ \] Add
export preview and data inclusion summary. - \[ \] Add path redaction
options. - \[ \] Add import preview and conflict resolution. - \[ \] Add
schema upgrade path for imported bundles. - \[ \] Add backup before
destructive import operations. - \[ \] Add complete project data
deletion command and UI. - \[ \] Add data inspection command. - \[ \]
Test round-trip export/import. - \[ \] Test large archives and malformed
input. - \[ \] Document format compatibility guarantees.

Deliverables: - Stable export/import formats. - Backup, restore,
inspect, and delete workflows.

Exit criteria: - Export/import round-trip preserves record identity and
content. - User can understand and control every destructive operation.

### Phase 8 --- Quality, accessibility, and release readiness

**Goal:** Make the MVP reliable enough for public use.

Tasks: - \[ \] Test Windows, Linux, and macOS. - \[ \] Test supported VS
Code versions. - \[ \] Test projects with spaces and non-ASCII paths. -
\[ \] Test read-only workspaces. - \[ \] Test network drives where
supported. - \[ \] Test large Git repositories. - \[ \] Test no-Git
folders. - \[ \] Test multi-root workspaces. - \[ \] Test database lock
and recovery behavior. - \[ \] Test upgrade from previous app
versions. - \[ \] Audit logs for accidental sensitive data. - \[ \]
Audit all file and process operations. - \[ \] Review keyboard
navigation and screen-reader labels. - \[ \] Review color contrast and
theme compatibility. - \[ \] Measure startup and command latency. - \[
\] Write troubleshooting guide. - \[ \] Prepare changelog and release
notes. - \[ \] Prepare Marketplace listing and screenshots. - \[ \] Tag
a release candidate and collect feedback.

Deliverables: - Release candidate. - User documentation. - Security and
privacy review. - Release checklist.

Exit criteria: - No known data-loss issue. - No critical or
high-severity security issue. - CI and release checklist pass. -
Installation, upgrade, backup, and uninstall behavior are documented.

### Phase 9 --- Public MVP release

**Goal:** Publish a useful, trustworthy open-source product.

Tasks: - \[ \] Publish source repository. - \[ \] Publish VS Code
extension. - \[ \] Publish CLI binaries for supported platforms. - \[ \]
Publish documentation and examples. - \[ \] Add sample project data. -
\[ \] Add issue templates and support guidelines. - \[ \] Announce the
project with a concise demo. - \[ \] Triage early feedback. - \[ \]
Track bugs and usability friction without collecting telemetry by
default. - \[ \] Prioritize fixes based on reproducible user reports.

Deliverables: - Public release. - Reproducible release artifacts. -
Public contribution process.

Exit criteria: - Users can install, initialize, use, export, and recover
their project data using published documentation.

## 7. Post-MVP roadmap

Do not start these until the MVP is stable and users validate the core
workflow.

### 7.1 Optional assisted capture

-   [ ] Detect meaningful Git events and suggest a timeline entry.
-   [ ] Detect dependency manifest changes and offer a summary.
-   [ ] Detect build configuration changes and offer a context entry.
-   [ ] Suggest linking a decision to files or commits.
-   [ ] Allow review before saving every suggestion.
-   [ ] Provide per-collector enable/disable controls.
-   [ ] Keep raw file contents and secrets out of capture.
-   [ ] Add an audit view showing why each item was suggested.

### 7.2 Better project onboarding

-   [ ] Generate a factual project inventory.
-   [ ] Identify README, manifests, and build entry points.
-   [ ] Detect languages and tools using explainable evidence.
-   [ ] Generate a project map with source citations/paths.
-   [ ] Let users approve generated project notes.
-   [ ] Never invent architectural intent.

### 7.3 Integrations

-   [ ] GitHub issue and pull request links (no required account).
-   [ ] GitLab links.
-   [ ] Jira/Linear references as optional adapters.
-   [ ] Terminal integrations through explicit user actions.
-   [ ] Additional editor clients.
-   [ ] Public plugin/adapter API.

### 7.4 Optional AI provider interface

Only consider AI after local search and structured records are
excellent.

-   [ ] Define a provider-neutral interface.
-   [ ] Make AI disabled by default.
-   [ ] Require explicit user selection of context to send.
-   [ ] Show the exact data and destination before sending.
-   [ ] Support local models where feasible.
-   [ ] Never make AI required for core functionality.
-   [ ] Label generated suggestions and require user confirmation.

### 7.5 Team and synchronization possibilities

-   [ ] Evaluate Git-based Markdown synchronization.
-   [ ] Define conflict and merge semantics.
-   [ ] Consider optional encrypted sync only if there is clear demand.
-   [ ] Keep local-first functionality complete without sync.
-   [ ] Do not introduce mandatory accounts.

## 8. Non-functional requirements

### Performance targets

Initial targets (validate and adjust with real fixtures): - Core CLI
startup: under 200 ms for common commands on a typical development
machine, excluding cold OS startup and large repository scans. - Search:
under 300 ms for a dataset of 10,000 records on a typical development
machine. - Extension activation: avoid expensive project scans during
activation. - Database writes: transactional and responsive; long
imports must show progress. - Git history scan: bounded by configurable
depth and cancellable.

### Reliability

-   Atomic writes where practical.
-   Database migrations are transactional.
-   Cancellation support for long-running operations.
-   Clear errors with recovery guidance.
-   No silent data deletion.
-   Backups before destructive operations.

### Compatibility

-   Windows, Linux, and macOS.
-   x64 and ARM64 where the Rust toolchain and dependencies support
    them.
-   VS Code stable as the initial editor target.
-   No dependency on a specific shell.
-   No dependency on a specific Git hosting provider.

## 9. Testing strategy

-   Unit tests for domain validation and use cases.
-   Storage integration tests against temporary SQLite databases.
-   Migration tests from every released schema.
-   Import/export round-trip tests.
-   Fuzz or property-based tests for untrusted import data where
    practical.
-   CLI integration tests.
-   VS Code extension host tests.
-   Cross-platform CI.
-   Manual exploratory tests for multi-root, read-only, large, and
    unusual-path workspaces.
-   Privacy tests that verify secret-like values and environment
    variable values are not collected.
-   Performance tests with synthetic large datasets.

## 10. Definition of done

A task is done only when: - Implementation is complete. - Relevant tests
are added or updated. - Formatting and lint checks pass. - Errors and
edge cases are handled. - User-facing behavior is documented. - Privacy
implications are reviewed. - No unrelated scope is added. -
`PROGRESS.md` is updated with evidence. - Any architectural decision is
recorded when it affects future compatibility.

## 11. Release acceptance checklist

-   [ ] Fresh install works on Windows.
-   [ ] Fresh install works on Linux.
-   [ ] Fresh install works on macOS.
-   [ ] Project initialization works in Git and non-Git folders.
-   [ ] Notes can be created, edited, searched, exported, and deleted.
-   [ ] Decisions can be created, superseded, searched, and exported.
-   [ ] Timeline distinguishes authored records from Git metadata.
-   [ ] References open or clearly report unavailable targets.
-   [ ] Import preview and conflict resolution work.
-   [ ] Backup and restore are tested.
-   [ ] Data deletion is explicit and reliable.
-   [ ] No telemetry or network requirement exists.
-   [ ] Documentation explains storage and privacy.
-   [ ] CLI and extension protocol compatibility is tested.

## 12. How to use this roadmap

-   Treat this file as the **scope and sequencing reference**.
-   Do not mark tasks complete in this file.
-   Track task status, evidence, blockers, and dates in `PROGRESS.md`.
-   If scope changes, update this roadmap and record the reason in
    `CHANGELOG.md` or an ADR.
-   Do not skip phase exit criteria simply to move to the next phase.
