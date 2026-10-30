# Quickstart and Common Workflows

This guide walks through using **DevContext** from project initialization to day-to-day knowledge capture, search, references, and disaster recovery.

---

## 1. What DevContext Is (and Is Not)

- **What it is:** A local-first, privacy-by-default developer memory system. It preserves developer-authored notes, Architectural Decision Records (ADRs), milestones, references, development logs, and a unified chronological timeline combining Git commit history with engineering rationale.
- **What it is not:**
  - It is **not** an automated surveillance tool (it never sniffs arbitrary terminal inputs or scans private source files).
  - It is **not** an AI code generator or cloud-hosted database.
  - It is **not** a replacement for Git (Git tracks *how* code changed; DevContext records *why* decisions were made).

---

## 2. Project Initialization

### CLI Workflow
Navigate to your repository or project folder and initialize DevContext:

```bash
# In the current directory
devcontext init

# Or explicitly naming the project
devcontext init --name "Aurora Workflow Engine"
```

This creates an isolated `.devcontext/` directory containing:
- `devcontext.db`: SQLite database with WAL mode enabled.
- `devcontext.db-wal` / `devcontext.db-shm`: SQLite WAL auxiliary files.
- `.devcontext/backups/`: Directory for automatic safety snapshots.
- Automatically appends `.devcontext/` to `.git/info/exclude` so project memory is kept local and isolated from Git.

### VS Code Extension Workflow
1. Open your workspace folder in VS Code.
2. In the Activity Bar, click the **DevContext** container icon.
3. In the **Project Overview** view, click **Initialize DevContext**.

---

## 3. Capturing Developer Notes

Notes capture technical context, debugging findings, system architecture, or onboarding guidelines.

### CLI Workflow
```bash
# Add a note with title, content, tags, and category
devcontext note add "Task Dispatch Loop Priority" \
  -c "Worker threads consume tasks from ring buffer slots without cross-thread lock contention." \
  -C "architecture" \
  -t "scheduling" -t "core"

# List notes
devcontext note list

# Show note details
devcontext note show <note-id>

# Edit a note
devcontext note edit <note-id> -c "Updated body content"

# Delete a note (soft-deletes from active views)
devcontext note delete <note-id>
```

### VS Code Extension Workflow
- Press `Ctrl+Alt+D N` (macOS: `Cmd+Alt+D N`) or click the `+` icon in the **Developer Notes** view.
- Fill in the interactive prompts: Title, Content, Tags, and Category.
- Click any note in the tree view to view its details, tags, and linked references.

---

## 4. Architectural Decision Records (ADRs)

ADRs capture technical decisions, context, chosen solutions, and trade-offs.

### CLI Workflow
```bash
# Add an Architectural Decision
devcontext decision add "ADR-001: Adopt Embedded SQLite for Task Persistence" \
  --context "Need durable task persistence without external daemon infrastructure." \
  --decision "Adopt embedded SQLite with Write-Ahead Logging (WAL) mode." \
  --consequences "Zero-dependency single-file storage. Reads never block writers." \
  --alternatives "1. Plain JSON files (risk of corruption). 2. Postgres (external daemon)." \
  -t "storage" -t "sqlite"

# List decisions
devcontext decision list

# Show decision details
devcontext decision show <decision-id>

# Supersede an older decision with a newer decision ID
devcontext decision update <old-decision-id> \
  --status superseded \
  --superseded-by <new-decision-id>
```

### VS Code Extension Workflow
- Press `Ctrl+Alt+D A` (macOS: `Cmd+Alt+D A`) or click the `+` icon in the **Architecture Decisions** view.
- Follow the multi-step prompt: Title, Context, Decision, and Consequences.
- Right-click any existing ADR to select **Supersede Decision**.

---

## 5. Milestones and Development Logs

### CLI Workflow
```bash
# Add a target milestone
devcontext milestone add "v1.0 Dispatcher Stabilization" \
  --target-date "2026-11-15T00:00:00Z" \
  -d "Stabilize worker concurrency and fault recovery routines."

# Mark milestone completed
devcontext milestone complete <milestone-id>

# Record a development log entry
devcontext log add "Resolved queue starvation under heavy load" \
  --summary "Implemented fair round-robin scheduling yielding CPU cycles after 5 high-priority batches." \
  -d "Under high burst traffic, default tasks were delayed. Interleaving resolved starvation." \
  -t "bugfix" -t "concurrency"
```

---

## 6. Fast Full-Text Search (SQLite FTS5)

DevContext includes a built-in SQLite FTS5 search engine with BM25 ranking, snippet extraction, and Turkish/Unicode character normalization.

### CLI Workflow
```bash
# Full-text search across all notes, ADRs, milestones, and logs
devcontext search "sqlite"

# Filter search results by record type
devcontext search "dispatcher" --type note
devcontext search "concurrency" --type decision

# Limit results and output machine-readable JSON
devcontext search "buffer" --limit 10 --json
```

### VS Code Extension Workflow
- Press `Ctrl+Alt+D S` (macOS: `Cmd+Alt+D S`) or run **DevContext: Full-Text Search Knowledge** in the Command Palette.
- Type in the debounced live QuickPick search box.
- Results display highlighted match snippets and record types. Press `Enter` to navigate directly to the selected note or ADR.

---

## 7. Unified Project Timeline

DevContext seamlessly interleaves Git commit history with developer-authored notes, decisions, and milestones.

### CLI Workflow
```bash
# Display chronological timeline (default 50 events)
devcontext timeline

# Filter timeline to decisions only
devcontext timeline --type decision

# Filter timeline to Git commits only
devcontext timeline --type commit

# Filter timeline by date range
devcontext timeline --since "2026-10-01" --until "2026-10-31"
```

### VS Code Extension Workflow
- Open the **Project Timeline** tree view in the DevContext container.
- Filter events using the inline filter icons.
- Click any Git commit event to open an interactive, syntax-colored diff view in VS Code!

---

## 8. Typed References

DevContext allows linking notes and decisions to concrete artifacts:

| Reference Type | Target Syntax | Action |
| :--- | :--- | :--- |
| **File** | `./src/dispatcher.rs` or `./src/dispatcher.rs#L42-L78` | Opens file directly at anchor line range. |
| **Folder** | `./crates/storage` | Opens folder in file explorer. |
| **Git Commit** | `c0ffee0123456789abcdef...` (40-char SHA) | Opens Git commit diff editor in VS Code. |
| **External URL** | `https://example.com/spec` | Opens URL safely in external browser. |

In VS Code:
- Highlight code in the active editor, right-click, and select **DevContext: Create Note from Selection** to automatically anchor the note to the selected lines!

---

## 9. Export, Import, and Disaster Recovery

DevContext guarantees complete data portability and zero vendor lock-in.

### Differentiating Formats
- **JSON (`--format json`):** Interchange format conforming to `schemas/export-v1.schema.json`.
- **Markdown (`--format markdown`):** Standalone human-readable directory hierarchy (`decisions/`, `notes/`, `milestones/`, `logs/`, `commands/`, `README.md`) with YAML frontmatter.
- **ZIP (`--format zip`):** Portable single-file archive packaging JSON v1 alongside Markdown files.

### CLI Workflow
```bash
# Export project context
devcontext export --format json -o export.json
devcontext export --format markdown -o ./docs/context/
devcontext export --format zip -o project-memory.zip

# Preview export (record counts and path redaction check)
devcontext export --preview

# Inspect an external export bundle before importing
devcontext inspect project-memory.zip

# Import with dry-run verification (does not modify database)
devcontext import project-memory.zip --dry-run

# Import with conflict resolution strategy
devcontext import project-memory.zip --on-conflict merge
devcontext import project-memory.zip --on-conflict overwrite

# Safe project data deletion (automatically creates pre-delete backup in .devcontext/backups/)
devcontext delete-data --force
```

---

## 10. Self-Diagnostics (Doctor)

To verify the integrity of your DevContext installation and database:

```bash
devcontext doctor
```

Output verifies:
- Core executable version and git adapter status.
- SQLite WAL mode and foreign key constraint enforcement.
- Database file size and schema migration version.
- Record counts and FTS5 search index consistency.
