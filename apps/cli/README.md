# devcontext-cli

[![Crates.io](https://img.shields.io/crates/v/devcontext-cli.svg)](https://crates.io/crates/devcontext-cli)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/Taiizor/DevContext/blob/develop/LICENSE)
[![GitHub](https://img.shields.io/badge/github-Taiizor%2FDevContext-blue?logo=github)](https://github.com/Taiizor/DevContext)

**The official command-line interface for [DevContext](https://github.com/Taiizor/DevContext) — the local-first project memory system for software development.**

---

## What is DevContext?

Git records **how** code changed over time. DevContext preserves the human and technical rationale that explains **why** the project evolved:

- 🏛️ **Architecture Decisions (ADRs):** Explicit decision records with context, decision, consequences, and superseding history.
- 📝 **Developer Notes:** Focused rationale, architecture caveats, work logs, and domain learnings.
- 🎯 **Milestones & Logs:** Project milestone tracking and daily progress entries.
- 🔍 **Full-Text Search (FTS5):** Instant offline full-text search across all notes, decisions, and commands using SQLite FTS5 with BM25 ranking.
- ⏱️ **Unified Timeline:** Combined view of Git commits and developer context in strict chronological order.
- 🔒 **Local-first & Private:** Operates entirely offline on your local disk (`.devcontext/`). Zero telemetry, zero accounts, no network calls, and no mandatory AI.

---

## Installation

### Via Cargo (Recommended)

If you have the Rust toolchain installed:

```bash
cargo install devcontext-cli
```

This compiles and installs the `devcontext` executable to your Cargo binary directory (e.g. `~/.cargo/bin`). Verify with:

```bash
devcontext --version
# Output: devcontext 0.1.0
```

### Pre-built Binaries

Pre-compiled standalone executables for Windows, Linux, and macOS are available on the [GitHub Releases](https://github.com/Taiizor/DevContext/releases) page.

---

## Quickstart & Common Commands

### 1. Initialize DevContext in your project
```bash
devcontext init
```
*Creates an isolated `.devcontext/` directory in the project root with an embedded SQLite database and automatic `.gitignore` rules.*

### 2. Record an Architectural Decision (ADR)
```bash
devcontext decision add "ADR-001: Adopt Embedded SQLite with WAL" \
  -c "Need zero-dependency persistent storage across process restarts." \
  -d "Adopt embedded SQLite with Write-Ahead Logging." \
  --consequences "Single-file durability with zero external services." \
  -t "storage" -t "sqlite"
```

### 3. Capture a Developer Note
```bash
devcontext note add "Task Dispatch Loop Priority" \
  -c "Worker threads consume tasks from ring buffer slots without lock contention." \
  -C "architecture" \
  -t "scheduling" -t "core"
```

### 4. Search Project Memory (SQLite FTS5 + BM25)
```bash
devcontext search "sqlite"
```

### 5. View Unified Chronological Timeline
```bash
devcontext timeline
```

### 6. Track Milestones
```bash
devcontext milestone add "MVP Release" --due 2026-11-15
devcontext milestone list
```

### 7. Run Health Diagnostics
```bash
devcontext doctor
```

### 8. Export & Backup Project Memory
```bash
# Export to portable ZIP bundle
devcontext export --format zip --output ./backup.zip

# Export to versioned JSON
devcontext export --format json --output ./devcontext-export.json
```

---

## Global CLI Flags & JSON Mode

DevContext CLI is designed for both human terminal usage and automation:

```bash
# JSON output for tooling and scripts
devcontext --json status
devcontext --json search "sqlite"

# Explicit project root
devcontext --project-root /path/to/repo status
```

---

## Ecosystem

- **[devcontext-core](https://crates.io/crates/devcontext-core):** The core Rust domain library and persistence engine.
- **[VS Code Extension](https://marketplace.visualstudio.com/items?itemName=Taiizor.devcontext-memory):** VS Code sidebar tree views, QuickPick search, and editor commands (`Taiizor.devcontext-memory`).
- **[Documentation & User Guides](https://github.com/Taiizor/DevContext/tree/develop/docs/user):** Comprehensive guides on disaster recovery, privacy, and workflows.

---

## License

Dual-licensed under either:
- Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/Taiizor/DevContext/blob/develop/LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](https://github.com/Taiizor/DevContext/blob/develop/LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
