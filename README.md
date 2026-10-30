# DevContext

**Project memory for software development.**

DevContext is a local-first, privacy-by-default development memory system. It preserves the human and technical context behind a software project: technical decisions (ADRs), developer notes, milestones, references, selected Git history, and reusable commands.

Git records *how* files changed. DevContext records the human and technical context that explains *why* the project evolved.

---

## Status

DevContext is at **v0.1.0 (Public MVP Release)** (**Phase 9 — Public MVP Release**).

- Architecture decisions: [`docs/decisions/`](docs/decisions/)
- Product requirements: [`docs/requirements/product-requirements.md`](docs/requirements/product-requirements.md)
- User Documentation: [`docs/user/`](docs/user/)
  - [Quickstart & Common Workflows](docs/user/quickstart-and-workflows.md)
  - [Installation & Upgrade Guide](docs/user/installation-and-upgrade.md)
  - [Troubleshooting & Diagnostics](docs/user/troubleshooting.md)
  - [Backup & Disaster Recovery](docs/user/backup-and-recovery.md)
  - [Privacy & Storage Architecture](docs/user/privacy-and-storage.md)
- Sample Project Dataset: [`examples/sample-project/`](examples/sample-project/)
- Release Documentation: [`docs/release/`](docs/release/)
  - [Release Notes v0.1.0](docs/release/release-notes-v0.1.0.md)
  - [Public Announcement Draft](docs/release/public-announcement.md)
  - [Demo Script & Storyboard](docs/release/demo-script.md)
  - [Feedback & Triage Guide](docs/release/feedback-and-triage-guide.md)
- Community & Support: [`.github/SUPPORT.md`](.github/SUPPORT.md)
- Changelog: [`CHANGELOG.md`](CHANGELOG.md)
- Roadmap: [`ROADMAP.md`](ROADMAP.md)
- Live tracker: [`PROGRESS.md`](PROGRESS.md)

---

## User Quick Start

### 1. Initialize DevContext in your project
```bash
devcontext init
```

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

### 4. Search Your Project Memory (SQLite FTS5)
```bash
devcontext search "sqlite"
```

### 5. Inspect the Unified Timeline
```bash
devcontext timeline
```

---

## Product Principles

- **Local-first and offline-capable:** Operates entirely on your local machine with zero network dependencies.
- **No accounts or hosted services:** No signups, no cloud sync, no tracking.
- **No AI requirement:** Fully functional and deterministic without LLMs or remote models.
- **Human-owned knowledge:** Explanations and decisions are explicitly authored or approved by developers.
- **Portable data:** Portable Markdown directories, versioned JSON v1 schema, and ZIP bundles.
- **Editor-independent core:** Native Rust core with standalone CLI and VS Code client.
- **Privacy by default:** Zero telemetry, zero automated terminal or environment harvesting.

---

## Workspace Architecture

```text
DevContext/
├── Cargo.toml               # Root Rust workspace manifest
├── apps/
│   ├── cli/                 # Standalone native CLI binary (Rust)
│   └── vscode-extension/    # VS Code editor extension (TypeScript / Bun)
├── crates/
│   └── devcontext-core/     # Core domain logic, SQLite storage, search, git adapter
├── schemas/
│   └── export-v1.schema.json# Versioned export interchange schema
├── docs/
│   ├── architecture/        # Architecture documentation & diagrams
│   ├── decisions/           # Architectural Decision Records (ADRs)
│   └── requirements/        # Product requirements & acceptance checklists
└── fixtures/                # Testing fixtures and sample repositories
```

---

## Development Prerequisites

- **Rust:** 1.80+ (`cargo`, `rustc`, `rustfmt`, `clippy`)
- **Bun:** 1.1+ (TypeScript execution, package management, and packaging)
- **Git:** 2.30+

---

## Quick Start (Developer Setup)

### Building the Rust Workspace

```bash
# Check formatting
cargo fmt --check

# Run linter
cargo clippy --all-targets --all-features -- -D warnings

# Build all workspace crates
cargo build

# Run core and CLI test suite
cargo test
```

### Developing the VS Code Extension

```bash
cd apps/vscode-extension

# Install dependencies using Bun
bun install

# Type-check TypeScript code
bun run typecheck

# Run extension tests
bun test
```

---

## License

DevContext is dual-licensed under either:

- **MIT License** ([`LICENSE-MIT`](LICENSE-MIT))
- **Apache License, Version 2.0** ([`LICENSE-APACHE`](LICENSE-APACHE))

at your option.
