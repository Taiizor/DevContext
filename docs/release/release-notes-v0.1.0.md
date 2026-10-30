# DevContext Release Notes — Version 0.1.0-rc1

**Release Candidate 1 (v0.1.0-rc1)**
*Release Date: 2026-10-31*

DevContext is a local-first, privacy-by-default development memory system designed to preserve the human and technical context behind software projects.

> **"Git records *how* files changed. DevContext records *why*."**

---

## Highlights of Release Candidate 1

### 1. Robust Local-First SQLite Engine
- **Transactional Persistence:** SQLite storage configured with Write-Ahead Logging (WAL) and foreign-key integrity.
- **Atomic Migrations:** Forward-only transactional schema migrations with automatic upgrade verification.
- **Safe Recovery:** Automated safety snapshots, atomic backup creation with collision retry, and disaster recovery.

### 2. Standalone Native CLI (`devcontext`)
- Zero-dependency native executable for Windows, Linux, and macOS.
- Complete subcommands for project initialization, developer notes, architecture decisions, milestones, logs, timeline, search, doctor diagnostics, and export/import.
- Machine-readable `--json` output envelope for programmatic tool integration.

### 3. Visual Studio Code Extension
- Native TreeView providers for Project Overview, Developer Notes, Architecture Decisions (ADRs), and Timeline.
- QuickPick full-text search with live BM25 ranking, snippets, and keyboard shortcut (`Ctrl+Alt+D S` / `Cmd+Alt+D S`).
- Context actions: create notes from selection, link files, and open Git commit diffs directly in the editor.
- Non-blocking asynchronous operations with full accessibility labels for screen readers.

### 4. Portable Knowledge and Data Safety
- **Triple Export Formats:** Machine-readable versioned JSON (`export-v1.json`), human-readable standalone Markdown hierarchies, and portable compressed ZIP archives.
- **Privacy by Default:** Machine-specific absolute paths are automatically relativized or redacted. Zero environment variable harvesting.
- **Archive Security:** Zip Slip path traversal defenses, ADS colon and null byte rejection.
- **Pre-Operation Backups:** Automatic non-destructive backups before imports and project data deletions.

---

## Supported Platforms and Architectures

| Platform | Architecture | CLI Artifact | VS Code Extension Artifact |
| :--- | :--- | :--- | :--- |
| **Windows** | x86_64 | `devcontext.exe` | `devcontext-0.1.0-win32-x64.vsix` |
| **Linux** | x86_64 / aarch64 | `devcontext` | `devcontext-0.1.0-linux-x64.vsix` |
| **macOS** | Apple Silicon / Intel | `devcontext` | `devcontext-0.1.0-darwin-arm64.vsix` |

---

## Known Limitations and Operational Boundaries

- **Single Active Writer per Database:** While SQLite WAL mode supports concurrent readers without blocking, concurrent writes serialize automatically via `PRAGMA busy_timeout=5000`.
- **Non-Networked by Design:** DevContext does not offer remote cloud synchronization or multi-user server hosting in the MVP.
- **Saved Commands Safety:** Saved commands are stored as passive documentation metadata only and are never executed automatically.
