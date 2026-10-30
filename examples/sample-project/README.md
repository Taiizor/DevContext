# Sample Project: Aurora Workflow Engine

This directory provides a realistic, non-sensitive sample dataset for **DevContext**.

It demonstrates how developer notes, Architectural Decision Records (ADRs), milestones, development logs, saved commands, and references look in a real software project.

> [!NOTE]
> This dataset uses completely fictional project information ("Aurora Workflow Engine").
> It contains **no credentials, no personal paths, no private source code, and no live machine state**.

---

## Files in this Directory

- `aurora-sample-export.json`: A standard, versioned export bundle conforming to `schemas/export-v1.schema.json`.

---

## How to Use the Sample Data

### 1. Inspect without Importing

You can inspect the contents and record counts of the sample bundle offline without modifying any project:

```bash
devcontext inspect examples/sample-project/aurora-sample-export.json
```

Or for machine-readable JSON output:

```bash
devcontext inspect examples/sample-project/aurora-sample-export.json --json
```

Expected output:
- Project Name: `Aurora Workflow Engine`
- Schema Version: `1.0.0`
- Total Records: `7` (Notes: 1, Decisions: 2, Milestones: 2, Log Entries: 1, Saved Commands: 1)

---

### 2. Import into an Isolated Test Workspace

To explore this data safely without modifying your real projects, create an empty temporary directory:

```bash
# 1. Create a temporary folder
mkdir test-aurora-workspace
cd test-aurora-workspace

# 2. Preview the import (dry-run mode does not modify the database)
devcontext import ../examples/sample-project/aurora-sample-export.json --dry-run

# 3. Import the sample bundle
devcontext import ../examples/sample-project/aurora-sample-export.json
```

---

### 3. Explore the Imported Context

Once imported, you can run any DevContext command:

```bash
# View project status and statistics
devcontext status

# List developer notes
devcontext note list

# Show Architecture Decision Records (ADRs)
devcontext decision list
devcontext decision show <decision-id>

# Search the knowledge base using SQLite FTS5 (BM25 ranking)
devcontext search "sqlite"
devcontext search "dispatcher"
devcontext search "concurrency"

# View the unified project timeline
devcontext timeline
```

---

### 4. Safely Remove the Sample Data

When you are done testing:

```bash
# Option A: Delete all context records using DevContext CLI (creates an automatic backup snapshot)
devcontext delete-data --force

# Option B: Delete the temporary test folder entirely
cd ..
rmdir /s /q test-aurora-workspace  # On Windows PowerShell: Remove-Item -Recurse test-aurora-workspace
```
