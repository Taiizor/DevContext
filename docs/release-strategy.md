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
1. **CLI Standalone Binaries:**
   - Windows x64: `devcontext-vX.Y.Z-windows-x64.zip` (`x86_64-pc-windows-msvc`)
   - Windows ARM64: `devcontext-vX.Y.Z-windows-arm64.zip` (`aarch64-pc-windows-msvc`)
   - Linux x64: `devcontext-vX.Y.Z-linux-x64.tar.gz` (`x86_64-unknown-linux-gnu`, glibc 2.35+ baseline)
   - Linux ARM64: `devcontext-vX.Y.Z-linux-arm64.tar.gz` (`aarch64-unknown-linux-gnu`)
   - macOS Intel: `devcontext-vX.Y.Z-macos-x64.tar.gz` (`x86_64-apple-darwin`, macOS 11.0+)
   - macOS Apple Silicon: `devcontext-vX.Y.Z-macos-arm64.tar.gz` (`aarch64-apple-darwin`, macOS 11.0+)
2. **Platform-Specific VS Code Extension Packages (VSIX):**
   - Windows x64: `devcontext-memory-X.Y.Z-win32-x64.vsix`
   - Windows ARM64: `devcontext-memory-X.Y.Z-win32-arm64.vsix`
   - Linux x64: `devcontext-memory-X.Y.Z-linux-x64.vsix`
   - Linux ARM64: `devcontext-memory-X.Y.Z-linux-arm64.vsix`
   - macOS Intel: `devcontext-memory-X.Y.Z-darwin-x64.vsix`
   - macOS Apple Silicon: `devcontext-memory-X.Y.Z-darwin-arm64.vsix`
3. **Checksum Verification File:**
   - `SHA256SUMS.txt` listing SHA-256 hashes for all 12 release artifacts.
4. **Release Notes:**
   - Summarizing changes, deprecations, migration guides, and SHA-256 checksums.

### Distribution & Publishing Policy
- Standalone CLI binaries and VSIX packages are built, validated, and uploaded to a GitHub Release Draft via GitHub Actions.
- Visual Studio Marketplace publishing is strictly **MANUAL** and managed by authorized publishers using `bunx @vscode/vsce publish --packagePath <vsix>`.
- CI/CD pipelines never publish to Visual Studio Marketplace and require no Marketplace secrets (`VSCE_PAT`).

---

## 3. Tagging & Changelog Convention

- Git tags follow the format `vX.Y.Z` (e.g., `v0.1.0`).
- Releases are documented in a top-level `CHANGELOG.md` following [Keep a Changelog](https://keepachangelog.com/) standards.
