# ADR 0004: Multi-root workspace and workspace isolation

- **Status:** Accepted
- **Date:** 2026-10-30

## Context

In modern development workflows (particularly in VS Code), developers frequently open multi-root workspaces where several distinct repositories or sub-projects reside side-by-side in a single window.

If DevContext were to merge all open workspace folders into a single monolithic project or database, several failure modes would occur:
1. Records from one repository would leak into unrelated projects.
2. Cross-linking relative file paths would break when navigating between folders.
3. Repositories with distinct Git remotes would have mangled timelines.

## Decision

1. **Strict Workspace Isolation:**
   - Every initialized workspace root maintains its own discrete `.devcontext/` directory and SQLite database.
   - DevContext will **never silently merge multiple roots** into a single database.

2. **VS Code Multi-Root Behavior:**
   - When a multi-root workspace is detected in VS Code, DevContext displays each initialized folder as an independent top-level project container in the DevContext view container.
   - Commands (e.g. `DevContext: New Note`, `DevContext: Search`) operate on the active editor's corresponding project root. If no file is active, the extension prompts the user via a quick-pick to select which workspace root to target.

3. **CLI Behavior:**
   - The CLI discovers the project root by walking upward from the current working directory (`cwd`) until finding a `.devcontext/` directory or a `.git` root.
   - An explicit `--project-dir <PATH>` or `--root <PATH>` flag is supported on all CLI commands to override auto-discovery.

## Consequences

### Positive
- Strict data separation and no accidental data leakage between distinct repositories.
- Deterministic path resolution: all relative paths are anchored directly to that root.
- Clean mental model for the user.

### Trade-offs
- Searching across multiple disconnected repositories simultaneously requires an intentional multi-root aggregation feature, which is deferred to post-MVP.
