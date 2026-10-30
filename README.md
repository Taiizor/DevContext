# DevContext

**Project memory for software development.**

DevContext is a local-first, privacy-by-default development memory system. It preserves the human and technical context behind a software project: technical decisions (ADRs), developer notes, milestones, references, selected Git history, and reusable commands.

Git records *how* files changed. DevContext records the human and technical context that explains *why* the project evolved.

---

## Status

DevContext is currently in active development (**Phase 1 — Repository and Engineering Foundation**).

- Architecture decisions: [`docs/decisions/`](docs/decisions/)
- Product requirements: [`docs/requirements/product-requirements.md`](docs/requirements/product-requirements.md)
- Roadmap: [`ROADMAP.md`](ROADMAP.md)
- Live tracker: [`PROGRESS.md`](PROGRESS.md)

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
