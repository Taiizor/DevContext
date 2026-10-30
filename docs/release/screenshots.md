# Marketplace Screenshots Specification and Checklist

This document defines the specification and checklist for promotional screenshots required for the VS Code Marketplace listing and repository documentation.

---

## 1. Technical Requirements

- **Resolution:** 1280 x 800 px (16:10 aspect ratio) or 1920 x 1080 px (16:9 aspect ratio).
- **Format:** PNG (optimized, 24-bit RGB or 32-bit RGBA).
- **Themes:** Primary screenshots in **Default Dark+** (VS Code standard dark). Alternate high-contrast or light theme captures where appropriate.
- **DPI:** 100% or 200% (Retina / High-DPI).
- **Font:** Default VS Code fonts (Consolas, Menlo, Cascadia Code, or Fira Code).

---

## 2. Screenshot Checklist

### Screenshot 1: Activity Bar & Project Overview TreeView
- **File Name:** `01-overview-and-activity-bar.png`
- **Caption:** DevContext Activity Bar container showing Project Overview, Developer Notes, Architecture Decisions, and Timeline TreeViews.
- **Content:**
  - DevContext icon in the Activity Bar.
  - Project Overview item showing project name, record counts, and database health.
  - Notes tree showing pinned notes with tag badges.
  - Architecture Decisions tree showing status badges (`[ACCEPTED]`, `[PROPOSED]`, `[SUPERSEDED]`).

### Screenshot 2: Blazing-Fast Full-Text Search (QuickPick)
- **File Name:** `02-full-text-search.png`
- **Caption:** Fast BM25 full-text search with highlight snippets and record filtering.
- **Content:**
  - QuickPick search modal triggered via `Ctrl+Alt+D S`.
  - Query: `SQLite WAL`.
  - Results showing icon badges (`$(note)`, `$(law)`), titles, highlighted matching snippets, and tag details.

### Screenshot 3: Architecture Decision Record (ADR) Detail
- **File Name:** `03-architecture-decision-detail.png`
- **Caption:** Comprehensive Architecture Decision view displaying context, decision, consequences, and alternatives considered.
- **Content:**
  - Formatted markdown preview of an ADR (e.g. `ADR-0013: SQLite WAL Storage Engine`).
  - Superseding relationship notice linking to a previous decision.

### Screenshot 4: Unified Chronological Timeline & Git Diff Action
- **File Name:** `04-unified-timeline-and-git.png`
- **Caption:** Unified timeline interleaving Git commits with authored notes and milestones.
- **Content:**
  - Timeline view displaying commits and notes in chronological order.
  - Editor displaying real Git commit diff opened via `devcontext.openReference`.

### Screenshot 5: Export / Import Preview and Data Safety Modal
- **File Name:** `05-export-import-preview.png`
- **Caption:** Explicit modal preview showing record counts, automatic backup notices, and path redaction guarantees.
- **Content:**
  - Modal dialog asking for confirmation before export or import.
  - Summary of notes, decisions, and milestones.
  - Clear privacy explanation highlighting that passwords and absolute paths are redacted.
