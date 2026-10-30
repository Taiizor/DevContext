# ADR 0010: Process-Based Read-Only Git Adapter and Ephemeral Timeline Assembly

- **Status:** Accepted
- **Date:** 2026-10-30

## Context

Phase 4 requires integrating Git metadata into DevContext to present a unified chronological timeline combining Git commits with authored DevContext records (notes, decisions, milestones, logs).

Two implementation strategies were evaluated:
1. **Embedding `libgit2` (via `git2` crate):**
   - *Pros:* Native in-process C-library binding; does not rely on a system `git` binary.
   - *Cons:* Heavyweight native compilation dependency (`cmake`, OpenSSL, zlib); significantly inflates binary size and compile times; can experience platform-specific build failures across Windows/Linux/macOS toolchains; potential memory safety hazards at C boundary.
2. **Process-Based Git CLI Execution (`std::process::Command`):**
   - *Pros:* Completely zero third-party dependencies; uses the exact same Git configuration, credentials, and ignore mechanisms the developer already uses; lightweight and secure; zero C-compilation overhead; easy to mock or gracefully degrade when Git is not installed.
   - *Cons:* Requires `git` to be installed on the system `$PATH` for Git-specific features.

Furthermore, we must decide whether to replicate/cache Git commits inside the local SQLite database or assemble the timeline ephemerally on-the-fly.

## Decision

1. **Process-Based Read-Only Adapter:**
   - DevContext will use a dedicated `GitAdapter` that queries the system `git` binary using `std::process::Command`.
   - If `git` is not installed or the directory is not a Git repository, DevContext degrades gracefully without errors—Git features are simply bypassed, and DevContext records are displayed normally.

2. **Strict Read-Only Enforcement:**
   - The adapter will **only** invoke read-only querying commands:
     - `git rev-parse --is-inside-work-tree`
     - `git rev-parse --show-toplevel`
     - `git rev-parse --abbrev-ref HEAD` (branch name)
     - `git rev-parse HEAD` (head commit)
     - `git log --format=...` (bounded commit history)
     - `git cat-file -e <hash>` (commit existence check)
   - Mutating commands (`commit`, `branch`, `checkout`, `reset`, `stash`, `merge`, `config`, `hook`) are strictly prohibited.

3. **Ephemeral Timeline Assembly (No Commit Duplication):**
   - DevContext will **not** store or sync full Git commit histories into the SQLite database.
   - The timeline is assembled dynamically in memory: DevContext records are queried from SQLite, Git commits are boundedly read from `git log`, merged into a unified `TimelineEvent` stream, sorted chronologically by UTC timestamp, and paginated.
   - This prevents database bloat in large repositories (e.g., repos with 100,000+ commits) and ensures Git metadata is always fresh and never desynchronized.

## Consequences

### Positive
- Zero external crate dependencies added to `Cargo.toml`.
- Extremely fast compilation and minimal binary size.
- Safe by default: physically impossible to mutate Git state.
- No stale Git cache: timeline always reflects real-time repository state.
- Graceful degradation in non-Git workspaces.

### Trade-offs
- Git timeline integration requires a working `git` binary installed on `$PATH`. In environments without Git, timeline displays only authored DevContext records.
