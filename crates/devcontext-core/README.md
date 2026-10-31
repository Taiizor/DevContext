# devcontext-core

[![Crates.io](https://img.shields.io/crates/v/devcontext-core.svg)](https://crates.io/crates/devcontext-core)
[![Docs.rs](https://docs.rs/devcontext-core/badge.svg)](https://docs.rs/devcontext-core)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/Taiizor/DevContext/blob/develop/LICENSE)
[![GitHub](https://img.shields.io/badge/github-Taiizor%2FDevContext-blue?logo=github)](https://github.com/Taiizor/DevContext)

**Core domain logic, persistence engine, and search primitives for [DevContext](https://github.com/Taiizor/DevContext) — the local-first project memory system for software development.**

---

## Overview

`devcontext-core` provides the fundamental data models, embedded storage layer, full-text search, Git adapters, and export/import primitives used by the DevContext ecosystem. 

It is designed with strict product principles:
- **Local-first & offline:** Completely self-contained with zero network calls, accounts, or telemetry.
- **Embedded persistence:** Pure embedded SQLite 3 with Write-Ahead Logging (WAL) and atomic transactional migrations.
- **High-performance search:** Native SQLite FTS5 full-text search with BM25 relevance ranking and Unicode normalization.
- **Human-owned knowledge:** Dedicated domain structures for Developer Notes, Architectural Decisions (ADR), Milestones, and References.
- **Data safety:** Built-in automatic pre-operation backups, Zip Slip archive protections, and path traversal guards.

---

## Installation

Add `devcontext-core` to your `Cargo.toml`:

```toml
[dependencies]
devcontext-core = "0.1"
```

---

## Quick Example

```rust
use devcontext_core::domain::{Decision, DecisionStatus, Note, Project};
use devcontext_core::search::{SearchFilter, SearchService};
use devcontext_core::storage::SqliteStorage;
use uuid::Uuid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Open an isolated in-memory storage (or use SqliteStorage::open("path/to/devcontext.db"))
    let storage = SqliteStorage::open_in_memory()?;

    // 2. Initialize a project record
    let project_id = Uuid::new_v4();
    let project = Project::new(project_id, "my-service");
    storage.insert_project(&project)?;

    // 3. Record an Architectural Decision (ADR)
    let decision = Decision::new(
        Uuid::new_v4(),
        project_id,
        "ADR-001: Adopt Embedded SQLite with WAL",
        "Need zero-dependency persistent storage across process restarts.",
        "Adopt embedded SQLite with Write-Ahead Logging for atomic transactions.",
        "Single-file durability, crash resilience, and zero external services.",
        DecisionStatus::Accepted,
    );
    storage.insert_decision(&decision)?;

    // 4. Capture a Developer Note
    let note = Note::new(
        Uuid::new_v4(),
        project_id,
        "Task Dispatch Loop Priority",
        "Worker threads consume tasks from ring buffer slots without lock contention.",
        Some("architecture".to_string()),
        false,
    );
    storage.insert_note(&note)?;

    // 5. Query Project Memory using Full-Text Search (SQLite FTS5 + BM25)
    let search_service = SearchService::new(storage);
    let results = search_service.search("sqlite", &SearchFilter::default())?;

    for result in results {
        println!("[{}] {} (score: {:.2})", result.record_type, result.title, result.score);
    }

    Ok(())
}
```

---

## Architecture & Modules

The crate is structured into focused modules:

| Module | Description |
|---|---|
| [`domain`](https://docs.rs/devcontext-core/latest/devcontext_core/domain/index.html) | Rich domain entities: `Project`, `Note`, `Decision` (ADR lifecycle & history), `Milestone`, `LogEntry`, `SavedCommand`, and `Reference`. |
| [`storage`](https://docs.rs/devcontext_core/latest/devcontext_core/storage/index.html) | Embedded SQLite persistence with WAL mode, foreign key enforcement, migration runners, and backup services. |
| [`search`](https://docs.rs/devcontext-core/latest/devcontext_core/search/index.html) | SQLite FTS5 search service with BM25 ranking, title weighting, and Turkish/Unicode case-folding. |
| [`git`](https://docs.rs/devcontext-core/latest/devcontext_core/git/index.html) | Strictly read-only Git adapter for commits, branch status, and unified chronological timeline generation. |
| [`export`](https://docs.rs/devcontext-core/latest/devcontext_core/export/index.html) | Versioned JSON (v1 schema) export/import, portable ZIP bundles, conflict resolution, and path sanitization. |
| [`reference`](https://docs.rs/devcontext-core/latest/devcontext_core/reference/index.html) | Deterministic resolver for file paths, Git commit SHAs, symbols, and URLs without network access. |
| [`workspace`](https://docs.rs/devcontext-core/latest/devcontext_core/workspace/index.html) | Project root detection, workspace initialization, and multi-root isolation. |

---

## Ecosystem

- **[devcontext-cli](https://crates.io/crates/devcontext-cli):** The official standalone CLI binary (`devcontext`).
- **[VS Code Extension](https://marketplace.visualstudio.com/items?itemName=Taiizor.devcontext-memory):** Native tree views, QuickPick search, and editor integrations (`Taiizor.devcontext-memory`).
- **[GitHub Repository](https://github.com/Taiizor/DevContext):** Source code, issues, roadmaps, and architecture decision records (ADRs).

---

## License

Dual-licensed under either:
- Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/Taiizor/DevContext/blob/develop/LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](https://github.com/Taiizor/DevContext/blob/develop/LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
