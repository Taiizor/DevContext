# Troubleshooting Guide

This guide helps diagnose and resolve common issues encountered while using DevContext CLI and the VS Code extension.

---

## 1. Core Executable Resolution Issues

### Symptom: "DevContext core is not available" or "Executable not found"
The VS Code extension cannot locate a compatible native `devcontext` binary.

### Resolution:
1. **Verify Core Locator Resolution Hierarchy:**
   DevContext resolves the core executable using three layered sources:
   - **Custom Configuration:** Check VS Code settings:
     `devcontext.core.executablePath`: Set this to the absolute path of the built binary (e.g. `C:/target/release/devcontext.exe` or `/usr/local/bin/devcontext`).
   - **Bundled Platform Binary:** Ensure the platform-specific VSIX was installed for your operating system and architecture (`win32-x64`, `linux-x64`, `darwin-arm64`).
   - **System PATH:** Ensure `devcontext` is in your system `PATH`. Open a terminal and run `devcontext --version`.
2. **Run Diagnostic Doctor:**
   - In VS Code: Open Command Palette (`Ctrl+Shift+P` / `Cmd+Shift+P`) and run `DevContext: Run Doctor Diagnostic`.
   - In CLI: Run `devcontext doctor --json` in the project root.

---

## 2. Incompatible Core Version Handshake

### Symptom: "Incompatible protocol version"
The extension and CLI have mismatched major protocol versions.

### Resolution:
DevContext uses semantic versioning for IPC communication.
- If the core executable is v0.1.x and the extension requires v1.x, update both the CLI and extension to matching release versions.
- Running `devcontext doctor` reports the active CLI version and protocol compatibility.

---

## 3. Database Locking and Concurrency

### Symptom: `SQLITE_BUSY` or "Database lock poisoned"
Multiple processes or threads are contending for write access, or an unhandled process termination held a lock.

### Resolution:
1. **SQLite WAL Mode:** DevContext automatically configures `PRAGMA journal_mode=WAL` and `PRAGMA busy_timeout=5000`. Writes wait up to 5 seconds before reporting busy.
2. **Close Stale CLI Processes:** Ensure no background CLI process or hanging terminal session is holding an exclusive transaction.
3. **Check SQLite Sidecars:** Check `.devcontext/devcontext.db-wal` and `.devcontext/devcontext.db-shm`. Do not manually delete these files while a process is running; SQLite checkpoints them automatically.

---

## 4. Read-Only Workspaces

### Symptom: "Cannot initialize / import in a read-only workspace"
DevContext blocks any write or mutation operation in untrusted or read-only workspaces.

### Resolution:
- Verify workspace permissions: Ensure your user account has write permissions to the workspace directory and `.devcontext/`.
- VS Code Workspace Trust: In untrusted workspaces, DevContext operates in strict read-only mode to prevent unintended state changes. Grant workspace trust to enable mutations.

---

## 5. Corruption Recovery and Emergency Snapshots

### Symptom: "Integrity check failed" or "Emergency safety snapshot preserved"
A database file failed SQLite's `PRAGMA integrity_check` or an interrupted operation triggered emergency preservation.

### Resolution:
1. **Inspect Integrity:**
   Run `devcontext doctor` to view database health.
2. **Locate Emergency Snapshots:**
   Emergency snapshots are saved under `.devcontext/backups/emergency-manual-recovery-<timestamp>-<uuid>.db`.
3. **Manual Restoration:**
   - Back up the existing `.devcontext/devcontext.db`.
   - Copy the verified emergency snapshot file to `.devcontext/devcontext.db`.
   - Run `devcontext doctor` to verify that integrity check passes (`"ok"`).

---

## 6. Git Commit Reference Issues

### Symptom: "Commit hash not found in local repository"
Opening a Git commit reference fails or reports that the commit does not exist.

### Resolution:
- **Shallow Clones:** If the repository is a shallow clone (`git rev-parse --is-shallow-repository`), older commits may not be present locally. Run `git fetch --unshallow` to retrieve complete history.
- **Detached HEAD or Rebased Commits:** If history was rewritten (e.g. `git rebase` or `git commit --amend`), the referenced SHA may be unreachable. DevContext preserves the original reference metadata for historical documentation.
