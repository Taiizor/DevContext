# DevContext

**Project memory for software development.**

DevContext is a local-first tool that preserves the context behind a
software project: technical decisions, developer notes, milestones,
references, selected Git history, and reusable commands.

Git records how files changed. DevContext records the human and
technical context that helps explain why the project evolved.

## Status

DevContext is currently in the planning stage. The roadmap and agent
instructions are being established before implementation.

## Documents

-   [`ROADMAP.md`](ROADMAP.md) --- product scope, phases, deliverables,
    and acceptance criteria.
-   [`PROGRESS.md`](PROGRESS.md) --- live implementation tracker and
    verification log.
-   [`AGENTS.md`](AGENTS.md) --- instructions for coding agents and
    contributors.

## Product principles

-   Local-first and offline-capable.
-   No account or hosted service required.
-   No AI requirement.
-   Human-authored and user-approved knowledge.
-   Portable Markdown and versioned JSON exports.
-   Independent core with VS Code and CLI clients.
-   Privacy and data safety by default.

## Planned architecture

-   Rust core and CLI.
-   TypeScript VS Code extension.
-   SQLite local storage.
-   Full-text search.
-   Versioned JSON and Markdown export.

Implementation details may change through documented architecture
decisions.

## Planned capabilities

-   Project overview and identity.
-   Notes and technical decision records.
-   Milestones and development log.
-   References to files, symbols, commits, URLs, and commands.
-   Searchable project timeline.
-   Git metadata integration.
-   Backup, export, and import.
-   VS Code extension and standalone CLI.

## Contributing

The contribution process will be finalized before the public MVP. Until
then, use `AGENTS.md` and `ROADMAP.md` as the project planning
references.
