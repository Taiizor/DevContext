# DevContext

[![CI](https://github.com/Taiizor/DevContext/actions/workflows/ci.yml/badge.svg)](https://github.com/Taiizor/DevContext/actions/workflows/ci.yml)
[![Crates.io devcontext-cli](https://img.shields.io/crates/v/devcontext-cli?label=devcontext-cli&logo=rust)](https://crates.io/crates/devcontext-cli)
[![Crates.io devcontext-core](https://img.shields.io/crates/v/devcontext-core?label=devcontext-core&logo=rust)](https://crates.io/crates/devcontext-core)
[![VS Code Marketplace](https://img.shields.io/visual-studio-marketplace/v/Taiizor.devcontext-memory?label=VS%20Code&logo=visual-studio-code)](https://marketplace.visualstudio.com/items?itemName=Taiizor.devcontext-memory)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange?logo=rust)](https://www.rust-lang.org)

**Project memory for software development.**

DevContext is a local-first, privacy-by-default development memory system. It preserves the human and technical context behind a software project: technical decisions (ADRs), developer notes, milestones, references, selected Git history, and reusable commands.

Git records *how* files changed. DevContext records the human and technical context that explains *why* the project evolved.

---

## Status

DevContext is at **v0.1.2** (**Six-Platform VSIX Distribution Architecture**).

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
  - [Release Notes](docs/release/release-notes-v0.1.0.md)
  - [Public Announcement Draft](docs/release/public-announcement.md)
  - [Demo Script & Storyboard](docs/release/demo-script.md)
  - [Feedback & Triage Guide](docs/release/feedback-and-triage-guide.md)
- Community & Support: [`.github/SUPPORT.md`](.github/SUPPORT.md)
- Changelog: [`CHANGELOG.md`](CHANGELOG.md)
- Roadmap: [`ROADMAP.md`](ROADMAP.md)
- Live tracker: [`PROGRESS.md`](PROGRESS.md)

---

## Installation

### VS Code Extension (Recommended)
Install **DevContext by Taiizor** (`Taiizor.devcontext-memory`) from the [Visual Studio Marketplace](https://marketplace.visualstudio.com/items?itemName=Taiizor.devcontext-memory) or via command line:
```bash
code --install-extension Taiizor.devcontext-memory
```
*Zero-configuration out of the box:* The extension package automatically includes the pre-compiled native `devcontext` engine for your platform (`win32-x64`, `win32-arm64`, `linux-x64`, `linux-arm64`, `darwin-x64`, or `darwin-arm64`). **No separate CLI installation is required.**

### Standalone CLI (Optional / Headless Environments)
If using DevContext purely in terminal or CI environments without VS Code:
```bash
cargo install devcontext-cli
```
*Pre-compiled standalone binaries for all six supported architectures are also available on [GitHub Releases](https://github.com/Taiizor/DevContext/releases).*

#### Platform Compatibility Baseline
- **Windows:** Windows 10/11 x64 and ARM64.
- **Linux:** Linux distributions with glibc 2.35+ (Ubuntu 22.04+, Debian 12+, Fedora 36+, RHEL 9+).
- **macOS:** macOS 11.0+ (Big Sur, Monterey, Ventura, Sonoma, Sequoia) on Intel and Apple Silicon.
  - *Gatekeeper Note:* Installing via the VS Code Marketplace UI operates seamlessly. If manually installing a downloaded `.vsix` on macOS, remove any browser quarantine attribute if prompted: `xattr -d com.apple.quarantine devcontext-memory-*.vsix`.

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
