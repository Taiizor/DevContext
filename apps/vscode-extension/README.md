# DevContext for Visual Studio Code

**Local-first project memory system for software development. Preserve the "why" behind your code.**

Git records **how** your codebase changed. DevContext records **why**.

DevContext is a local-first, privacy-by-default developer memory extension. It preserves developer-authored notes, Architectural Decision Records (ADRs), milestones, references, and a unified chronological timeline combining Git commit history with engineering rationale.

---

## Key Features

- **Architectural Decision Records (ADRs):** Document technical decisions, proposed alternatives, and consequences. Track superseding relationships without rewriting history (`Ctrl+Alt+D A` / `Cmd+Alt+D A`).
- **Developer Notes:** Capture debugging findings, architectural notes, and project context with Markdown and tags (`Ctrl+Alt+D N` / `Cmd+Alt+D N`).
- **Full-Text Search (SQLite FTS5):** Blazing-fast BM25 full-text search with Turkish & Unicode normalization. Find any note or decision in milliseconds (`Ctrl+Alt+D S` / `Cmd+Alt+D S`).
- **Unified Chronological Timeline:** Interleaves Git commit history with authored notes and milestones. Filter by commits, notes, or decisions.
- **Typed References:** Link notes to files, line selections, Git commits, and URLs. Open files or Git commit diffs directly from tree views.
- **Durable Export & Import:** Export to versioned JSON (`export-v1.json`), clean Markdown hierarchies, or portable ZIP archives. Includes automatic pre-import safety backups and path redaction.
- **Zero Telemetry & 100% Offline:** Operates entirely on your local machine. No accounts, no hosted services, no network dependencies, no tracking.

---

## Keyboard Shortcuts

| Command | Windows / Linux | macOS |
| :--- | :--- | :--- |
| **DevContext: Full-Text Search** | `Ctrl+Alt+D S` | `Cmd+Alt+D S` |
| **DevContext: New Note** | `Ctrl+Alt+D N` | `Cmd+Alt+D N` |
| **DevContext: New Decision (ADR)** | `Ctrl+Alt+D A` | `Cmd+Alt+D A` |

---

## Zero-Configuration Bundled Architecture

DevContext is published to the Visual Studio Marketplace as **platform-specific VSIX packages** across six target architectures:
- `win32-x64` (Windows 64-bit Intel/AMD)
- `win32-arm64` (Windows on ARM64)
- `linux-x64` (Linux 64-bit, glibc 2.35+)
- `linux-arm64` (Linux ARM64, glibc 2.35+)
- `darwin-x64` (macOS Intel, 11.0+)
- `darwin-arm64` (macOS Apple Silicon, 11.0+)

When you install DevContext from the Marketplace, VS Code automatically downloads the package containing only the native `devcontext` engine compiled for your machine. **No manual installation of Rust, Cargo, or CLI binaries is required.**

### Layered Binary Resolution & Overrides

For advanced users and contributors, the extension locates the executable in the following strict priority sequence:

1. **Explicit User Setting:** `devcontext.core.executablePath` in VS Code Settings (overrides bundled binary).
2. **Bundled Platform Binary:** The pre-compiled native binary bundled at `bin/devcontext` or `bin/devcontext.exe`.
3. **Workspace Development Build:** `target/debug/` or `target/release/` in active repository workspace.
4. **System PATH:** Globally installed `devcontext` executable.

To test your core binary connection and health, run **DevContext: Run Doctor Diagnostic** (`devcontext.doctor`) from the Command Palette (`Ctrl+Shift+P`).

### Platform & OS Considerations
- **Linux:** Requires glibc 2.35 or newer (Ubuntu 22.04+, Debian 12+, Fedora 36+).
- **macOS:** Native arm64 and x86_64 binaries are compatible with macOS 11.0+. When installed from Marketplace, execution is seamless. For manual VSIX installations, ensure quarantine flags are cleared if prompted by Gatekeeper (`xattr -d com.apple.quarantine <vsix>`).

---

## Privacy Policy & Security

- **Local-first:** All databases reside in `<workspace>/.devcontext/`.
- **Zero data collection:** No telemetry, analytics, or remote API calls.
- **Untrusted data protection:** Imported bundles are checked against Zip Slip attacks and path traversals.
- **Saved commands safety:** Saved commands are stored as passive documentation and never executed automatically.

---

## Documentation & Support

- [Installation & Upgrade Guide](https://github.com/Taiizor/DevContext/blob/develop/docs/user/installation-and-upgrade.md)
- [Troubleshooting & Diagnostics](https://github.com/Taiizor/DevContext/blob/develop/docs/user/troubleshooting.md)
- [Backup & Disaster Recovery](https://github.com/Taiizor/DevContext/blob/develop/docs/user/backup-and-recovery.md)
- [Community Guidelines](https://github.com/Taiizor/DevContext/blob/develop/.github/SUPPORT.md)
- [GitHub Repository](https://github.com/Taiizor/DevContext)

---

## License

DevContext is dual-licensed under the **MIT** and **Apache-2.0** licenses.
