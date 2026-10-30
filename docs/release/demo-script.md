# DevContext Public Demo Script & Storyboard

This document defines the official, privacy-safe demo storyboard for **DevContext v0.1.0 MVP**.

It covers the complete genuine workflow required by Phase 9 using fictional sample data ("Aurora Workflow Engine"):
1. Initialize a project.
2. Create a developer note with file reference.
3. Record an architectural decision (ADR).
4. Full-text search across stored knowledge.
5. Inspect the unified chronological timeline.
6. Export project knowledge to Markdown and portable ZIP.

---

## Recording Guidelines

- **Environment:** Clean terminal window (80x24 or 120x30 font size) and VS Code with standard Dark+ theme.
- **Privacy:** Never show personal home paths (`C:\Users\username`), proprietary repos, private Git commits, or API keys.
- **Pacing:** Allow 2–3 seconds between commands for readability.

---

## Step-by-Step Demo Script

### Scene 1: Project Initialization (0:00 - 0:20)
**Action:** Open an empty or sample Git repository and initialize DevContext.

```bash
mkdir aurora-engine && cd aurora-engine
git init -q
git commit --allow-empty -m "chore: initial repository structure"

# Initialize DevContext
devcontext init --name "Aurora Workflow Engine"
```

**Voiceover / Caption:**
> "DevContext preserves the 'why' behind your software project. With `devcontext init`, a local SQLite database with Write-Ahead Logging is created inside `.devcontext/`. It is 100% offline, zero-telemetry, and automatically excluded from Git."

---

### Scene 2: Capturing Developer Notes with File Anchors (0:20 - 0:45)
**Action:** Add a technical note describing the core dispatch loop.

```bash
devcontext note add "Task Dispatch Loop Priority Bands" \
  -c "Worker threads consume tasks from ring buffer slots without cross-thread lock contention. Critical tasks are handled first." \
  -C "architecture" \
  -t "scheduling" -t "core"
```

**Voiceover / Caption:**
> "Capture technical findings, debugging context, or onboarding notes directly from your terminal or editor. Notes support Markdown, categories, and tags."

---

### Scene 3: Recording an Architecture Decision (ADR) (0:45 - 1:15)
**Action:** Document an Architectural Decision Record.

```bash
devcontext decision add "ADR-001: Adopt Embedded SQLite with WAL for Job Persistence" \
  -c "Aurora requires durable task persistence across process restarts without external database daemons." \
  -d "Adopt embedded SQLite with Write-Ahead Logging (WAL) and synchronous = NORMAL." \
  -C "Single-file durability with zero external services. Reads never block writers." \
  -a "1. Plain JSON files (risk of corruption). 2. PostgreSQL container (violates zero-dependency requirement)." \
  -t "storage" -t "sqlite"
```

**Voiceover / Caption:**
> "Architecture decisions explain why trade-offs were made. DevContext captures context, decision, consequences, and alternatives considered without rewriting commit history."

---

### Scene 4: Blazing-Fast Full-Text Search (1:15 - 1:40)
**Action:** Search the knowledge base using SQLite FTS5.

```bash
# Instant search across all notes, ADRs, milestones, and logs
devcontext search "sqlite"

# Live QuickPick in VS Code: Ctrl+Alt+D S
```

**Voiceover / Caption:**
> "Powered by SQLite FTS5 with BM25 ranking and Unicode normalization, search finds any note or decision in milliseconds with highlighted snippets."

---

### Scene 5: Unified Chronological Timeline (1:40 - 2:05)
**Action:** Display the combined development timeline.

```bash
# Unified timeline merging Git commits with authored context
devcontext timeline
```

**Voiceover / Caption:**
> "Git tracks how files changed. DevContext combines Git commits with authored notes and milestones into a single chronological timeline."

---

### Scene 6: Export & Portability (2:05 - 2:30)
**Action:** Export to human-readable Markdown and portable ZIP archive.

```bash
# Export to Markdown hierarchy
devcontext export --format markdown --out ./docs/context/

# Export to portable ZIP bundle
devcontext export --format zip --out aurora-context.zip
```

**Voiceover / Caption:**
> "Zero vendor lock-in. Export your project memory anytime to readable Markdown directories or portable ZIP bundles."

---

## Screenshot Specifications

| ID | View / Command | Purpose |
| :--- | :--- | :--- |
| `shot-01-overview` | VS Code DevContext Container | Shows Overview, Notes, Decisions, and Timeline trees. |
| `shot-02-search` | VS Code QuickPick Search (`Ctrl+Alt+D S`) | Shows live FTS5 query for `"sqlite"` with highlighted snippets. |
| `shot-03-adr-detail` | Editor showing ADR Markdown | Displays context, decision, consequences, and superseding status. |
| `shot-04-timeline` | Terminal `devcontext timeline` | Demonstrates Git commit interleaved with authored notes. |
| `shot-05-diff-editor` | Git Commit Diff Editor | Shows click-to-open Git commit diff from the timeline. |
| `shot-06-export` | Markdown Export Directory | Shows `decisions/`, `notes/`, `README.md` hierarchy. |
