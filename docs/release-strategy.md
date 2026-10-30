# Release Versioning Strategy

- **Status:** Approved (Phase 1)
- **Standard:** [Semantic Versioning 2.0.0 (SemVer)](https://semver.org/)

---

## 1. Version Numbering

DevContext components follow the format `MAJOR.MINOR.PATCH`:

- **MAJOR:** Incompatible API, CLI command behavior, database schema migrations without automatic upgrade, or protocol breaking changes.
- **MINOR:** Backwards-compatible new features, new reference types, or export schema additions.
- **PATCH:** Backwards-compatible bug fixes, performance improvements, or documentation updates.

### Pre-MVP Versioning (`0.y.z`)
During development (Phases 1 through 8):
- `0.1.0` represents the initial development baseline.
- Breaking changes may occur between minor versions prior to the Phase 9 Public MVP release (`1.0.0`).

---

## 2. Release Artifacts

Each release generates:
1. **CLI Binaries:**
   - Windows x64 & ARM64 (`devcontext-x86_64-pc-windows-msvc.zip`)
   - Linux x64 & ARM64 (`devcontext-x86_64-unknown-linux-gnu.tar.gz`)
   - macOS Intel & Apple Silicon (`devcontext-universal-apple-darwin.tar.gz`)
2. **VS Code Extension Package:**
   - `devcontext-vscode-X.Y.Z.vsix`
3. **Release Notes:**
   - Summarizing changes, deprecations, migration guides, and SHA-256 checksums.

---

## 3. Tagging & Changelog Convention

- Git tags follow the format `vX.Y.Z` (e.g., `v0.1.0`).
- Releases are documented in a top-level `CHANGELOG.md` following [Keep a Changelog](https://keepachangelog.com/) standards.
