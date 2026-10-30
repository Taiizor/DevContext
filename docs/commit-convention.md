# Commit Convention Specification

- **Status:** Approved (Phase 1)
- **Standard:** [Conventional Commits v1.0.0](https://www.conventionalcommits.org/en/v1.0.0/)

---

## 1. Format

Every commit message must follow this structure:

```text
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

## 2. Allowed Types

| Type | Description |
|---|---|
| `feat` | Introduces a new feature or domain capability. |
| `fix` | Resolves a defect or incorrect behavior. |
| `docs` | Additions or corrections to documentation or ADRs. |
| `style` | Whitespace, formatting, or missing semicolon changes (no logic changes). |
| `refactor` | Code restructuring without fixing a bug or adding a feature. |
| `perf` | Performance improvement. |
| `test` | Adding or updating unit/integration tests or test fixtures. |
| `chore` | Build script changes, package management, CI workflows, or toolchain updates. |

## 3. Recommended Scopes

- `core`: Code inside `crates/devcontext-core` (domain, storage, search, git, export).
- `cli`: Code inside `apps/cli`.
- `extension`: Code inside `apps/vscode-extension`.
- `schema`: Changes to `schemas/`.
- `docs`: Architecture, roadmap, or user documentation.
- `ci`: GitHub Actions workflows or local automation scripts.

## 4. Breaking Changes

Breaking changes must include an exclamation mark before the colon (`feat(cli)!: change flag syntax`) or contain a `BREAKING CHANGE:` footer.
