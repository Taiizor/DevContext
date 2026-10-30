# DevContext v0.1.0 — Public MVP Announcement Draft

> [!IMPORTANT]
> **Draft Notice:**
> This announcement is prepared for the upcoming public MVP release.
> Do **NOT** post or publish publicly without explicit maintainer approval.
> Links marked `[PENDING PUBLICATION]` will be active once publication gates are approved.

---

## Announcing DevContext: Local-First Project Memory for Software Development

Git records **how** files changed. DevContext records **why**.

We are excited to introduce **DevContext (v0.1.0 MVP)**: an open-source, local-first developer memory system designed to preserve technical decisions, architectural rationale, developer notes, milestones, and development context directly in your software repository.

---

### Why DevContext?

Software engineering produces an immense amount of valuable context:
- Why did we choose SQLite over Postgres for this service?
- What caused that elusive deadlock in the worker pool, and how did we resolve it?
- Which lines of code implement the task dispatch loop?

Too often, this context is scattered across fleeting Slack messages, external ticketing systems, closed wiki accounts, or lost inside someone's head. When a developer changes teams or a year passes, the "why" is lost.

DevContext brings that context back to where developers actually work:
1. **Local-First & 100% Offline:** Operates entirely on your local machine using an embedded SQLite database (`.devcontext/`). Zero cloud dependencies, zero external daemons, and zero accounts.
2. **Zero Telemetry:** DevContext respects developer privacy. No usage metrics, no tracking, no network calls, and no involuntary harvesting of your terminal or source code.
3. **Architectural Decision Records (ADRs):** Document technical decisions, alternatives considered, and consequences. Supersede older decisions without rewriting history.
4. **Blazing-Fast Search (SQLite FTS5):** BM25 ranking with Turkish and Unicode character normalization delivers sub-15ms search across thousands of records.
5. **Unified Project Timeline:** Combines Git commit history with authored notes and milestones to tell the complete story of your project's evolution.
6. **Complete Data Portability:** Export your project memory anytime to clean Markdown hierarchies, standard versioned JSON (`export-v1.json`), or portable ZIP archives. Includes automatic safety backups before destructive actions.

---

### Get Started in Seconds

#### CLI
Download pre-built standalone binaries for Windows, Linux, and macOS:
- Release Page: `https://github.com/DevContext/DevContext/releases/tag/v0.1.0` `[PENDING PUBLICATION]`

```bash
# Initialize DevContext in your project
devcontext init

# Record an architectural decision
devcontext decision add "ADR-001: Adopt SQLite with WAL" \
  -c "Need embedded, zero-dependency persistence." \
  -d "Adopt SQLite with Write-Ahead Logging." \
  --consequences "High concurrency, single-file durability."

# Search your project context
devcontext search "sqlite"
```

#### VS Code Extension
Install from the Visual Studio Code Marketplace:
- Marketplace URL: `https://marketplace.visualstudio.com/items?itemName=devcontext.devcontext` `[PENDING PUBLICATION]`
- Or install via VSIX: `code --install-extension devcontext-0.1.0.vsix`

Features include native tree views for Notes, Decisions, and Timeline, live QuickPick search (`Ctrl+Alt+D S`), and click-to-open Git commit diffs.

---

### Open Source & Community

DevContext is dual-licensed under the **MIT** and **Apache-2.0** licenses.
- Source Code: [https://github.com/DevContext/DevContext](https://github.com/DevContext/DevContext)
- Documentation & Guides: [docs/user/quickstart-and-workflows.md](../user/quickstart-and-workflows.md)
- Community Guidelines: [.github/SUPPORT.md](../../.github/SUPPORT.md)
