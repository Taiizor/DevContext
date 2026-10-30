# Contributing to DevContext

Thank you for your interest in contributing to DevContext! We are building a reliable, local-first developer memory system.

---

## 1. Ground Rules & Code of Conduct

- This project is governed by the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating, you agree to uphold this code.
- Always respect the core tenets in [`ROADMAP.md`](ROADMAP.md) and [`AGENTS.md`](AGENTS.md):
  - **Local-first and private by default:** Never introduce mandatory network calls, accounts, or telemetry.
  - **Human-owned knowledge:** Never generate arbitrary rationale without explicit user approval.
  - **No unconsented surveillance:** Never capture terminal input, arbitrary environment variables, or private source code silently.

---

## 2. Commit Message Convention

DevContext follows the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```text
<type>(<scope>): <short description>

[optional body]

[optional footer(s)]
```

### Allowed Types
- `feat`: A new user-facing feature.
- `fix`: A bug fix.
- `docs`: Documentation changes only.
- `refactor`: A code change that neither fixes a bug nor adds a feature.
- `test`: Adding missing tests or correcting existing tests.
- `chore`: Maintenance tasks, build configuration, or dependency updates.

### Examples
- `feat(core): add note search indexing support`
- `fix(cli): handle detached HEAD state in git adapter`
- `docs(adr): record ADR 0010 on attachment storage`

---

## 3. Development Workflow

### Prerequisites
- **Rust Toolchain:** Stable 1.80+ (`rustup default stable`)
- **Bun:** 1.1+ ([bun.sh](https://bun.sh))
- **Git:** 2.30+

### Working with the Rust Core and CLI
The Rust workspace is located at the repository root and managed via `Cargo.toml`.

```bash
# Check formatting
cargo fmt --check

# Run clippy with strict warnings
cargo clippy --all-targets --all-features -- -D warnings

# Run all unit and integration tests
cargo test

# Build debug binaries
cargo build
```

### Working with the VS Code Extension
The extension is located in `apps/vscode-extension` and managed exclusively with **Bun**.

```bash
cd apps/vscode-extension

# Install dependencies
bun install

# Verify TypeScript types
bun run typecheck

# Run test suite
bun test
```

---

## 4. Definition of Done for Contributions

Every pull request must satisfy:
1. All existing and new tests pass on Windows, Linux, and macOS.
2. `cargo fmt --check` and `cargo clippy --all-targets --all-features -- -D warnings` produce zero warnings or errors.
3. TypeScript code typechecks cleanly (`bun run typecheck`).
4. Any architectural change is documented as an ADR in `docs/decisions/`.
5. No credentials, secrets, or temporary local databases (`.devcontext/`) are checked in.
