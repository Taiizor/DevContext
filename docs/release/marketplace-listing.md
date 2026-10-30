# VS Code Marketplace Listing Text

## Title
DevContext

## Subtitle
Local-first project memory system for software development. Preserve the "why" behind your code.

## Description (Overview Tab)

Git records **how** your codebase changed. DevContext records **why**.

DevContext is a local-first, privacy-by-default development memory extension. It preserves developer-authored notes, Architectural Decision Records (ADRs), milestones, references, and a unified chronological timeline combining Git commit history with engineering rationale.

---

### Key Features

- **Architectural Decision Records (ADRs):** Document technical decisions, proposed alternatives, and consequences. Track superseding relationships without rewriting history.
- **Developer Notes:** Capture debugging findings, architectural notes, and project context with Markdown and tags.
- **Full-Text Search (SQLite FTS5):** Blazing-fast BM25 full-text search with Turkish & Unicode normalization. Find any note or decision in milliseconds (`Ctrl+Alt+D S` / `Cmd+Alt+D S`).
- **Unified Chronological Timeline:** Combines Git commit history with authored notes and milestones. Filter by commits, notes, or decisions.
- **Typed References:** Link notes to files, line selections, Git commits, and URLs. Open files or Git commit diffs directly from tree views.
- **Durable Export & Import:** Export to versioned JSON (`export-v1.json`), clean Markdown hierarchies, or portable ZIP archives. Includes automatic pre-import safety backups and path redaction.
- **Zero Telemetry & 100% Offline:** Operates entirely on your local machine. No accounts, no hosted services, no network dependencies, no tracking.

---

### Keyboard Shortcuts

| Command | Windows / Linux | macOS |
| :--- | :--- | :--- |
| **Full-Text Search** | `Ctrl+Alt+D S` | `Cmd+Alt+D S` |
| **New Note** | `Ctrl+Alt+D N` | `Cmd+Alt+D N` |
| **New Decision (ADR)** | `Ctrl+Alt+D A` | `Cmd+Alt+D A` |

---

### Privacy Policy & Security

- **Local-first:** All databases reside in `<workspace>/.devcontext/`.
- **Zero data collection:** No telemetry, analytics, or remote API calls.
- **Untrusted data protection:** Imported bundles are checked against Zip Slip attacks and path traversals.
- **Saved commands safety:** Saved commands are stored as passive documentation and never executed automatically.

---

### Categories
- `Productivity`
- `Other`

### Tags
`project-memory`, `adr`, `architecture-decisions`, `developer-notes`, `local-first`, `git`, `timeline`, `privacy`
