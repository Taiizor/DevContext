# Changelog

All notable changes to the DevContext project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.2] - 2026-10-31

### Added
- **Six-Platform VSIX Distribution Architecture:**
  - Introduced platform-specific VS Code extension packaging (`win32-x64`, `win32-arm64`, `linux-x64`, `linux-arm64`, `darwin-x64`, `darwin-arm64`) under the single canonical Marketplace identity (`Taiizor.devcontext-memory`).
  - Bundled the pre-compiled platform-native `devcontext` CLI binary directly inside each platform VSIX package (`extension/bin/devcontext` or `devcontext.exe`), enabling zero-configuration installation without separate CLI, Rust, or Cargo installation.
  - Layered executable resolution in `CoreLocator`: preserves priority for explicit user override (`devcontext.core.executablePath`), bundled binary, workspace development builds (`target/debug` or `target/release`), and system PATH fallback.
  - Bundled distribution E2E test suite (`bundled_extension_e2e.test.ts`) covering binary discovery, priority overrides, handshake, SQLite FTS5 search, and containment security.
- **Isolated Staging & Package Validation Pipeline:**
  - Replaced shared `bin/` packaging with isolated, temporary staging (`apps/vscode-extension/scripts/packageTarget.ts`) using OS `mkdtemp` to prevent binary cross-contamination.
  - Added standalone VSIX archive and binary validator (`scripts/validateVsix.ts`) checking central directory entries, magic byte headers (PE x64 `0x8664`, PE ARM64 `0xaa64`, ELF x64 `0x3e`, ELF ARM64 `0xb7`, Mach-O x64 `0x01000007`, Mach-O ARM64 `0x0100000c`), Unix executable permissions (`0755`), and license file staging.
  - Added CodeQL security scanning workflow (`.github/workflows/codeql.yml`) for Rust and TypeScript/JavaScript.

### Changed
- **Dual Licensing Packaging & Documentation:**
  - Added root `LICENSE` descriptor explaining the dual-license model under MIT and Apache-2.0, explicitly referencing `LICENSE-MIT` and `LICENSE-APACHE`.
  - Updated packaging scripts to stage and validate `LICENSE`, `LICENSE-MIT`, and `LICENSE-APACHE` inside all VSIX bundles.
- **Extension Host Test Portability:**
  - Removed machine-specific VS Code paths from `runExtensionHost.ts`.
  - Implemented multi-platform executable discovery respecting `VSCODE_PATH`, standard installation locations across Windows, Linux, and macOS, and falling back to `@vscode/test-electron` automated download.
- **CoreLocator Self-Healing Security Hardening:**
  - Added strict path containment, regular file validation, symlink rejection, and missing `X_OK` permission check before applying executable permissions exclusively to bundled binaries on Unix platforms.
- **Target Verification Status:**
  - Windows x64 (`win32-x64`): Fully verified locally end-to-end (compilation, packaging, PE header validation, bundled E2E tests, and real Extension Host execution).
  - Windows ARM64 (`win32-arm64`), Linux x64 (`linux-x64`), Linux ARM64 (`linux-arm64`), macOS Intel (`darwin-x64`), macOS Apple Silicon (`darwin-arm64`): Toolchain, cross-compiler, linker, glibc baseline (2.35+), and isolated staging configurations verified; binary execution verified via CI workflow matrix.

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
