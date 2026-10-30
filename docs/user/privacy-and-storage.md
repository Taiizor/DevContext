# Privacy and Storage Architecture

DevContext is built on the core principle of **Privacy by Default**. It is designed for engineers working on proprietary, sensitive, or air-gapped codebases.

---

## 1. Privacy Guarantees

- **Zero Telemetry:** DevContext contains zero tracking, analytics, usage statistics, or crash reporting pings.
- **Zero Mandatory Network Calls:** Core operations execute 100% offline. No cloud accounts, hosted APIs, or remote logins are required.
- **No Arbitrary Scanning:** DevContext never scans arbitrary source file trees or terminal input to extract memories heuristically.
- **No Environment Harvesting:** Environment variables (`$ENV`, `%PATH%`, secrets, tokens, credentials) are never automatically recorded or exported.
- **Human-Owned Knowledge:** Rationale, architectural decisions, and notes are created or approved explicitly by developers.

---

## 2. Storage & Git Boundary

DevContext isolates its SQLite storage from version control:
1. **Default Storage Directory:** Stored in `<project-root>/.devcontext/`.
2. **Git Exclusion:**
   - During `devcontext init`, a `.gitignore` file is automatically placed inside `.devcontext/` to exclude `devcontext.db`, `*-wal`, `*-shm`, and `backups/`.
   - Additionally, `.git/info/exclude` is updated if inside a Git repository, ensuring shared `.gitignore` files are never dirtied.
3. **Sharing Knowledge via Version Control:**
   To share decisions or notes with a team via Git, use `devcontext export --format markdown` to generate clean, human-readable Markdown directories (such as `docs/decisions/` or `notes/`) that can be committed to the repository.

---

## 3. Path Sanitization and Redaction

When exporting data (JSON or ZIP bundles):
- **Workspace Containment:** Paths pointing inside the project root are relativized (`src/main.rs`).
- **Machine-Specific Path Redaction:** Absolute paths pointing outside the workspace (e.g. `/Users/dev/...` or `C:\Users\dev\...`) are redacted by default with `[REDACTED_EXTERNAL_PATH]`.
- **Zip Slip Prevention:** All archive import and extraction routines reject paths with `..`, absolute drives (`C:`), colons (`:`), or null bytes.

---

## 4. Reusable Saved Commands Safety

DevContext allows developers to document reusable commands (e.g. build scripts, deployment snippets, migration commands):
- **Storage Only:** Commands are stored as passive text metadata.
- **Zero Automated Execution:** Saved commands are **NEVER** executed automatically as a side effect.
- **Explicit User Action:** Any execution requires an explicit, deliberate user action with confirmation.
