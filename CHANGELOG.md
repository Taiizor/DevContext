# Changelog

All notable changes to the DevContext project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.1] - 2026-10-31

### Changed
- **Crates.io Documentation & Manifests:**
  - Added rich, standalone `README.md` files for both `devcontext-core` and `devcontext-cli` with usage examples, module architecture maps, and absolute documentation links.
  - Resolved inter-crate dependency versioning and package metadata for crates.io publishing.
- **VS Code Extension Activity Bar Icon:**
  - Redesigned `media/icon.svg` as a dedicated 24x24 transparent monochrome vector icon (`currentColor`) for the VS Code Activity Bar, eliminating solid white squircle mask artifact.
  - Preserved full 256x256 colored branding vector in `media/logo.svg` and Marketplace PNG in `media/icon.png`.
- **Version Bump:**
  - Bumped Cargo workspace packages (`devcontext-core`, `devcontext-cli`) and VS Code extension to `v0.1.1`.

---

## [0.1.0] - 2026-10-31

### Added
- **Public MVP Release:**
  - Initial public release of DevContext (Rust core/CLI and VS Code extension).
  - Standalone native binary builds for Windows x64, Linux x64, and macOS (x64 / ARM64).
  - Production-ready VS Code extension package (`devcontext-memory-0.1.0.vsix`).
  - Shipped sample project dataset (`examples/sample-project/`) demonstrating notes, ADRs, milestones, logs, references, and search.
  - Comprehensive user documentation: Quickstart & Workflows, Installation & Upgrade, Troubleshooting, Backup & Recovery, and Privacy Architecture.
  - Public community and support guidelines (`.github/SUPPORT.md`) and refined issue templates with strict privacy warnings.
  - Automated release workflow (`.github/workflows/release.yml`) with SHA-256 checksum generation.
- **Core Storage Engine:**
  - Robust SQLite storage configured with Write-Ahead Logging (`WAL`), `PRAGMA busy_timeout=5000`, and `foreign_keys=ON`.
  - Transactional forward-only database schema migrations with idempotence checks.
  - Native domain entities: `Project`, `Record`, `Note`, `Decision`, `Milestone`, `LogEntry`, `SavedCommand`, and `Reference`.
- **Search & Indexing:**
  - Full-text search engine powered by SQLite FTS5 with BM25 ranking algorithm.
  - Custom Turkish character folding and case normalization (handling `İ`, `ı`, `Ş`, `ş`, `Ğ`, `ğ`, `Ü`, `ü`, `Ç`, `ç`, `Ö`, `ö`).
  - Search filtering by record type, status, tag, and query matching.
- **Git Integration & Timeline:**
  - Read-only Git adapter for commit metadata extraction without mutating user repository state.
  - Unified chronological timeline interleaving Git commits with authored notes, decisions, and milestones.
  - Support for non-Git folders, detached HEAD, shallow clones, and empty repositories.
- **Standalone Native CLI (`devcontext`):**
  - Zero-dependency native executable compiled in Rust.
  - Commands: `init`, `status`, `doctor`, `note`, `decision`, `milestone`, `log`, `timeline`, `search`, `export`, `import`, `inspect`, and `delete-data`.
  - Structured `--json` response envelope with standard exit codes (0 for success, 1 for domain errors, 2 for validation errors).
- **VS Code Extension:**
  - Native TreeView providers for Project Overview, Developer Notes, Architecture Decisions, and Project Timeline.
  - Real-time QuickPick full-text search with live highlighting snippets.
  - Editor context actions: create notes from selected text, link files to notes, open Git commit diffs.
  - Three-tiered core binary resolution: configured path, bundled platform binary, and system `PATH`.
  - Status bar indicator reflecting project status and core health.
  - Accessibility information labels on all tree items for screen reader navigation.
- **Data Safety, Portability & Disaster Recovery:**
  - Triple export formats: versioned JSON v1 (`export-v1.json`), standalone Markdown directory hierarchies, and compressed portable ZIP archives.
  - Mandatory automatic pre-import and pre-deletion backup snapshots in `.devcontext/backups/`.
  - Atomic exclusive backup file creation (`create_new(true)`) with collision retry loops.
  - Emergency safety snapshot preservation during failed database rollback.
  - Path sanitization relativizing internal files and redacting external machine-specific absolute paths.
  - Archive security validation preventing Zip Slip directory traversal, ADS colons, and null-byte injection.

### Security
- Verified zero telemetry, zero analytics, and zero network dependencies.
- Reusable saved commands are stored strictly as passive documentation metadata and are never executed automatically.
- Automatic Git exclusion isolating `.devcontext/` from shared version control repositories.
