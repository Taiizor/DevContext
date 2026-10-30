# ADR 0012: Rust Core Binary Distribution and Resolution Strategy

- **Status:** Accepted
- **Date:** 2026-10-30

## Context

The DevContext Visual Studio Code extension relies on the native Rust `devcontext` core executable to manage SQLite storage, enforce transactional integrity, query full-text search indexes, and read Git history.

Risk R01 (identified in `PROGRESS.md`) required evaluating how the VS Code extension discovers or provisions the compatible Rust core executable across supported operating systems (Windows, Linux, macOS) and architectures (x64, ARM64).

Three distribution models were evaluated:

1. **Runtime Download from GitHub Releases:**
   - *Pros:* Generates a single platform-agnostic `.vsix` package.
   - *Cons:* Strictly violates DevContext's core local-first and offline-first product principle. The extension fails in air-gapped environments, behind corporate proxies, or when GitHub API rate limits are exceeded. Furthermore, downloading and executing native binaries dynamically at runtime introduces severe security risks and triggers corporate endpoint detection (EDR) blocks.

2. **System PATH Only:**
   - *Pros:* Extension package contains only TypeScript code.
   - *Cons:* High friction for end users who install the extension from the VS Code Marketplace expecting an out-of-the-box experience.

3. **Platform-Targeted VSIX Packaging with Layered Resolution (Selected):**
   - *Pros:* Fully offline-first. Uses the official VS Code Marketplace platform-specific extensions feature (`vsce package --target win32-x64, linux-x64, darwin-arm64, etc.`). The platform-specific binary is bundled directly inside the installed extension package. In addition, users and developers can override the binary location via settings or system `PATH`.
   - *Cons:* Requires automated multi-target builds in release CI (scheduled for Phase 8/9).

## Decision

DevContext adopts a **layered resolution strategy** that strictly preserves local-first, offline-capable operation:

### 1. Layered Resolution Order
When activating or invoking the core, the extension locates the executable in the following deterministic sequence:

1. **User/Workspace Setting (`devcontext.core.executablePath`):**
   Explicit path configured by the developer. This allows local development builds, custom installations, and multi-version testing.
2. **Bundled Extension Binary (`bin/<platform>/devcontext[.exe]`):**
   The platform-specific binary bundled within the extension directory (e.g., `bin/win32-x64/devcontext.exe`, `bin/darwin-arm64/devcontext`, `bin/linux-x64/devcontext`).
3. **System `PATH` Lookup:**
   Standard system search for `devcontext` (or `devcontext.exe` on Windows), allowing globally installed binaries (e.g. installed via `cargo install devcontext-cli`, Homebrew, Winget, or APT).

### 2. Strict Offline Guarantee
Under no circumstances will the extension initiate network downloads of native executables at runtime. If no compatible executable is found:
- The extension does not crash.
- A prominent, actionable notification is displayed with diagnostic details and setup guidance.
- Views show a "DevContext Core Not Found" state with a button to run diagnostics or configure the path.

### 3. Version Handshake and Compatibility Verification
Whenever an executable is located:
- The extension executes `devcontext doctor --json` (or `devcontext --version --json`) with structured arguments.
- It parses the response envelope and verifies that the reported `protocol_version` matches the extension's supported major version (currently `"1.0"`).
- If the binary is incompatible or reports an error, the extension notifies the user and disables commands safely to prevent database corruption.

### 4. Release Asset Dependency and Current Phase Scope
- In **Phase 6 (Current MVP):** The extension implements the full layered discovery engine, version handshake, and diagnostic handling. For local development and extension host testing, the extension discovers the binary from `devcontext.core.executablePath`, target build output (`target/debug/devcontext` or `target/release/devcontext`), or system `PATH`.
- In **Phase 8/9 (Packaging & Public Release):** The release CI matrix will produce stripped release binaries for `win32-x64`, `win32-arm64`, `linux-x64`, `linux-arm64`, `darwin-x64`, and `darwin-arm64`, and package platform-targeted `.vsix` bundles via `vsce`.

## Consequences

### Positive
- **100% Local-First:** Functions without internet access, proxies, or external service dependencies.
- **Security:** No unverified dynamic code downloads or arbitrary script executions.
- **Developer-Friendly:** Easily pointed to debug/development binaries via settings.
- **Reliability:** Platform-specific VSIX packages ensure zero post-install setup for end users.

### Trade-offs
- CI packaging pipeline must build and sign binaries across matrix targets before publishing to Marketplace (Phase 8/9 requirement).
