# ADR 0007: Extension-to-core communication protocol and versioning

- **Status:** Accepted
- **Date:** 2026-10-30

## Context

The VS Code extension requires an efficient, reliable way to execute DevContext operations (creating notes, retrieving timeline items, querying search results) by communicating with the native Rust binary.

Executing unvalidated shell command strings creates security risks (command injection, quoting bugs across Windows/POSIX) and fragile error handling. Furthermore, version mismatches between extension and CLI could lead to subtle data corruption or unhandled errors.

## Decision

1. **Structured Invocation over CLI JSON Interface:**
   - The VS Code extension does **not** construct or execute arbitrary shell strings.
   - It invokes the `devcontext` executable directly using `child_process.spawn` with structured argument arrays:
     ```typescript
     spawn("devcontext", ["note", "list", "--project-dir", rootPath, "--json"], { shell: false })
     ```
   - For interactive querying, all commands support a `--json` output flag producing standardized, typed envelope outputs.
   - For long-running operations or high-frequency editor sessions, an optional `devcontext rpc` stdio mode (JSON-RPC 2.0 over stdin/stdout) may be introduced in Phase 6 to eliminate process spawn overhead.

2. **Standard Output Envelope:**
   All machine-readable CLI responses follow a strict envelope format:
   ```json
   {
     "success": true,
     "data": { ... },
     "error": null
   }
   ```
   Or on error:
   ```json
   {
     "success": false,
     "data": null,
     "error": {
       "code": "PROJECT_NOT_FOUND",
       "message": "No .devcontext directory found at ...",
       "details": null
     }
   }
   ```

3. **Version Handshake & Protocol Negotiation:**
   - During extension activation, the extension runs `devcontext --version --json` or `devcontext doctor --json`.
   - The core binary reports its semantic version and supported protocol version (e.g. `protocol_version: "1.0"`).
   - If the core executable is missing, non-executable, or has an incompatible major protocol version, the extension displays a clear diagnostic warning with troubleshooting steps and gracefully disables editor features without crashing.

4. **Security & Path Validation:**
   - All workspace paths passed as arguments are verified by the extension to be canonicalized absolute paths belonging to open workspace folders.
   - Core validates that the target path is inside allowed filesystem bounds before opening any database.

## Consequences

### Positive
- Immune to shell command injection attacks.
- Robust cross-platform execution on Windows, Linux, and macOS.
- Clean separation: VS Code extension remains a thin, reactive UI client.

### Trade-offs
- The core binary must be installed or bundled with the extension for end-users, requiring platform-specific binary distribution or user configuration in settings.
