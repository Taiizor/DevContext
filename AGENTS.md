# DevContext --- Agent Instructions

These instructions apply to the entire repository unless a deeper
`AGENTS.md` overrides them.

## 1. Mission

Build DevContext as a reliable, local-first project memory system. Its
purpose is to preserve and retrieve the development context behind a
software project: developer-authored notes, technical decisions,
milestones, references, selected Git metadata, and saved commands.

The product must remain useful without AI, network access, accounts, or
hosted services.

## 2. Source of truth and planning

Before making changes: 1. Read `README.md` if present. 2. Read
`ROADMAP.md`. 3. Read `PROGRESS.md`. 4. Read relevant architecture
documents and ADRs. 5. Inspect the existing implementation and tests
before proposing a new structure.

Rules: - `ROADMAP.md` defines product scope, sequencing, and acceptance
criteria. - `PROGRESS.md` is the live execution tracker. - Do not use
the roadmap checkboxes as the live progress tracker. - Update
`PROGRESS.md` after each meaningful work session. - Mark work complete
only after verification. - Record architectural changes as ADRs under
`docs/decisions/`. - Do not silently expand MVP scope.

## 3. Communication and implementation approach

-   Prefer small, reviewable, coherent changes.
-   Work phase by phase; respect phase exit criteria.
-   Do not implement future-phase features early unless required by a
    current acceptance criterion.
-   Explain assumptions when a requirement is ambiguous.
-   If a decision materially affects data compatibility, privacy,
    security, or public APIs, stop and document the decision before
    implementation.
-   Do not claim a command or test passed unless it was actually run and
    its result inspected.
-   When a toolchain or dependency is unavailable, report the limitation
    and provide the exact unverified check.
-   Avoid broad rewrites when a targeted change is sufficient.

## 4. Product principles

### Local-first

-   Core functionality must work offline.
-   Do not add mandatory accounts, hosted APIs, or remote services.
-   Do not add telemetry to the MVP.
-   Do not send project content over the network.

### Human-owned knowledge

-   A commit message is metadata, not proof of why a decision was made.
-   Never invent rationale, authorship, intent, or architectural facts.
-   AI-generated or heuristic suggestions, if ever added, must be
    labeled and require user approval.

### Privacy by default

-   Never collect environment variable values automatically.
-   Never collect arbitrary terminal input.
-   Never scan and store arbitrary source file contents as project
    memory.
-   Do not capture credentials, tokens, private keys, or secrets.
-   Treat command text and imported archives as untrusted data.
-   Keep machine-specific absolute paths out of exports by default.
-   Explain every automatic collector and make it disableable.
-   Never execute saved commands, imported content, hooks, or scripts
    automatically.

### Portability

-   Keep the core independent from VS Code.
-   Keep business logic out of the extension UI.
-   Use stable, versioned data formats.
-   Preserve user data across upgrades.
-   Support Windows, Linux, and macOS where dependencies allow.

## 5. Architecture rules

Preferred initial stack: - Rust for core and CLI. - TypeScript for the
VS Code extension. - SQLite for local storage. - SQLite FTS5 for
full-text search, with a documented fallback if unavailable. -
JSON-based, versioned protocol between extension and core.

Boundaries: - Domain logic belongs in the core. - Persistence belongs
behind storage interfaces. - Git access belongs in a Git adapter. - VS
Code API usage belongs in the extension. - CLI parsing and terminal
rendering belong in the CLI. - Export/import logic belongs in a
dedicated module. - UI code must not contain SQL or domain validation
rules.

Do not split the codebase into many crates/packages prematurely. Start
with a small, understandable workspace and extract modules when there is
a concrete boundary.

Do not make the extension execute arbitrary shell command strings.
Invoke the known core executable with structured arguments and validate
all paths.

## 6. Data and compatibility rules

-   Use stable IDs for persisted records.
-   Define and document timestamp representation.
-   Use database migrations for schema changes.
-   Migrations must be tested and should be transactional.
-   Never destructively rewrite user data without backup and explicit
    consent.
-   Imported data is untrusted: validate schema, types, sizes, paths,
    and identifiers.
-   Do not trust archive paths; prevent path traversal during
    extraction.
-   Avoid storing duplicate derived data unless it can be rebuilt.
-   Keep export schemas versioned and documented.
-   Add round-trip tests for import/export.
-   Preserve superseded decisions as history; do not overwrite them.

## 7. VS Code extension rules

-   Use the official VS Code Extension API.
-   Keep activation lightweight.
-   Avoid expensive scans during extension activation.
-   Use cancellation for long-running operations.
-   Do not block the extension host with synchronous work.
-   Handle missing, crashed, or incompatible core executables
    gracefully.
-   Support workspace folders deliberately; never merge multiple roots
    silently.
-   Respect read-only workspaces.
-   Use VS Code theme tokens and accessible labels.
-   Provide keyboard-accessible workflows.
-   Do not add a webview for a feature that can be implemented more
    simply with native tree views, quick picks, or input boxes.
-   If a webview is justified, keep its message protocol typed and
    validate incoming messages.

## 8. Rust rules

-   Use safe Rust by default.
-   Avoid `unsafe` unless strictly necessary; document and test every
    unsafe boundary.
-   Return actionable errors; do not panic on user input or filesystem
    conditions.
-   Use structured error types at module boundaries.
-   Avoid swallowing errors.
-   Use parameterized SQL.
-   Keep database transactions explicit.
-   Add unit tests for validation and domain logic.
-   Add integration tests for persistence, migrations, and
    import/export.
-   Run `cargo fmt --check`,
    `cargo clippy --all-targets --all-features -- -D warnings`, and
    relevant `cargo test` commands before declaring Rust work complete.

## 9. TypeScript rules

-   Enable strict TypeScript settings.
-   Avoid `any`; use typed interfaces and discriminated unions.
-   Validate data received from the core process.
-   Dispose event listeners, watchers, processes, and VS Code resources
    correctly.
-   Avoid global mutable state.
-   Keep command handlers thin; delegate to typed client services.
-   Add extension host tests for user-facing workflows.
-   Run formatter, linter, typecheck, and tests before declaring
    extension work complete.

## 10. Security checklist

Before merging features that touch files, processes, imports, or
exports, consider: - Path traversal and symlink behavior. - Untrusted
workspace content. - Command injection. - Secret leakage in logs and
exports. - Archive bombs and oversized payloads. - SQL injection. -
Database corruption and partial writes. - Unintended network activity. -
Accidental destructive operations. - Permission and read-only filesystem
failures.

Never execute content from a note, decision, imported bundle, or saved
command without an explicit user action and a clear confirmation step.

## 11. Testing and definition of done

A change is complete only when: - Acceptance criteria are met. -
Relevant tests are added or updated. - Formatting, linting, and type
checks pass. - Errors and edge cases are handled. - Documentation is
updated. - Privacy and security impact is considered. - No unrelated
feature scope is introduced. - `PROGRESS.md` is updated with
verification evidence.

For user-visible behavior, include a reproducible manual test when
automated tests are insufficient.

## 12. Git and repository hygiene

-   Do not commit databases, local caches, build outputs, secrets, or
    machine-specific settings.
-   Do not rewrite public Git history.
-   Do not change license or public package identity without explicit
    approval.
-   Keep commits focused and descriptive.
-   Do not create a release or publish packages unless explicitly asked.
-   Do not modify user configuration outside the repository without
    explicit approval.

## 13. First task for a new agent

Start by inspecting the repository. If it is empty, begin with Phase 0
and Phase 1 planning only. Do not jump directly into implementing the
entire product.

Produce: 1. A concise repository assessment. 2. A list of unresolved
product and architecture decisions. 3. A proposed initial directory
structure. 4. A Phase 0/1 task breakdown. 5. An updated `PROGRESS.md`.

Wait for user approval before making irreversible product decisions,
publishing, or introducing external services.
