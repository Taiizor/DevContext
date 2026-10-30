# ADR 0006: Monorepo workspace structure and technology stack boundaries

- **Status:** Accepted
- **Date:** 2026-10-30

## Context

DevContext consists of two primary operational components:
1. A core native engine and command-line interface (CLI).
2. A Visual Studio Code editor extension.

To keep maintenance simple, avoid premature crate fragmentation, and maintain clear architectural boundaries, we need an agreed directory structure and toolchain strategy.

Furthermore, per repository developer rules:
- **Rust** is used for the core engine and CLI.
- **TypeScript** is used for the VS Code extension.
- **Bun** is the exclusive JavaScript/TypeScript runtime and package manager (no Node.js/npm/yarn/pnpm).

## Decision

1. **Monorepo Directory Layout:**
   We adopt a disciplined, unfragmented monorepo structure:
   ```text
   DevContext/
   ├── Cargo.toml               # Rust workspace manifest
   ├── Cargo.lock
   ├── apps/
   │   ├── cli/                 # devcontext native CLI binary (Rust)
   │   └── vscode-extension/    # VS Code extension UI and client (TypeScript / Bun)
   ├── crates/
   │   └── devcontext-core/     # Reusable domain logic, SQLite storage, search, git adapter
   ├── schemas/
   │   └── export-v1.schema.json
   ├── docs/
   │   ├── architecture/
   │   ├── decisions/
   │   └── requirements/
   └── tests/                   # End-to-end integration fixtures
   ```

2. **Crate Boundary Policy:**
   - Instead of immediately creating 6–8 micro-crates, we start with **one core library crate** (`crates/devcontext-core`) and **one CLI binary crate** (`apps/cli`).
   - `devcontext-core` encapsulates domain entities, SQLite migrations, FTS5 indexing, Git adapters, and export serialization in clear internal modules (`domain`, `storage`, `search`, `git`, `export`).
   - Crates will only be extracted into dedicated crates (e.g. `crates/devcontext-storage`) when there is a proven need for independent compilation or external distribution.

3. **VS Code Extension Workspace:**
   - `apps/vscode-extension` contains TypeScript sources compiled and packaged using `bun`.
   - The extension relies solely on the VS Code Extension API and does not duplicate core business logic or directly execute SQLite queries.

## Consequences

### Positive
- Fast build times and simplified dependency tree.
- Direct alignment with the guideline to avoid premature crate fragmentation.
- Single source of truth for schema and tests.

### Trade-offs
- The monorepo combines two toolchains (Rust via `cargo` and TypeScript via `bun`). Build scripts must orchestrate both cleanly during CI and local packaging.
