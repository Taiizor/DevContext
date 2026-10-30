# ADR 0005: Stable project identity and clone detection

- **Status:** Accepted
- **Date:** 2026-10-30

## Context

Developers often rename folders, move repositories to new drive letters, or maintain multiple clones of the same repository (e.g., worktrees, separate branch checkouts).

If project identity is tied to an absolute filesystem path (e.g., `E:/MyProject/Mix/GitHub/DevContext`), moving or renaming the folder would orphan the stored project context. Conversely, if two different clones of a repository are initialized, they might either collide or duplicate knowledge without warning.

## Decision

1. **Stable Project Identifier:**
   - Every project is assigned an immutable UUIDv4 upon initialization (`project.id`).
   - The UUID is stored inside `.devcontext/devcontext.db` in the `projects` table and in all export manifests.
   - The project's display name defaults to the folder name, but can be updated without affecting the project identifier.

2. **VCS Root Fingerprinting:**
   - When initializing inside a Git repository, DevContext records a non-sensitive repository fingerprint:
     - The SHA of the root commit (initial commit in history), or
     - The normalized Git remote origin URL (if configured and sanitized of credentials).
   - This fingerprint allows DevContext to recognize when two separate directory paths belong to the same logical Git repository.

3. **Clone and Move Detection Behavior:**
   - **Moved/Renamed Directory:** Since `.devcontext/` is relative to the project root, simply moving the folder keeps `.devcontext/devcontext.db` intact with its existing UUID. DevContext updates its cached last-known path smoothly.
   - **New Clone Initialization:** If `devcontext init` is executed in a folder with an existing Git repository whose fingerprint matches an existing DevContext project known to the machine, DevContext prompts the user:
     - Option A: *Link to existing project* (import/sync context from existing archive or location).
     - Option B: *Initialize as an independent context* (generates a fresh UUID).
   - DevContext will **never overwrite an existing database or silently merge identities**.

## Consequences

### Positive
- Robust against filesystem folder renaming and path migrations.
- Detects multi-clone development environments and prevents inadvertent data collisions.
- Preserves referential integrity in exported JSON and Markdown.

### Trade-offs
- Git root commit discovery requires invoking git commands or reading git object refs during initialization. If a git repository has zero commits (fresh `git init`), fingerprinting falls back to purely UUID-based local identification until the first commit is made.
