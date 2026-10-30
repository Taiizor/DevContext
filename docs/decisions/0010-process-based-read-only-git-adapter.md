# ADR 0010: Process-Based Read-Only Git Adapter and Ephemeral Timeline Assembly

- **Status:** Accepted (Updated & Verified)
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
   - If `git` is not installed or the directory is not a Git repository, DevContext degrades gracefully on `ErrorKind::NotFound` and non-zero exit codes without errors—Git features are simply bypassed, and native DevContext records are displayed normally.

2. **Strict Command Allowlist and Process Isolation:**
   - The adapter will **only** invoke read-only querying commands:
     - `git rev-parse --is-inside-work-tree`
     - `git rev-parse --show-toplevel`
     - `git rev-parse --abbrev-ref HEAD` (branch name)
     - `git rev-parse HEAD` (head commit)
     - `git log --format=...` (bounded commit history)
     - `git cat-file -e <hash>` (commit existence check)
   - Mutating commands (`commit`, `branch`, `checkout`, `reset`, `stash`, `merge`, `config`, `hook`, `rebase`) are strictly prohibited in the codebase.
   - Every subprocess invocation enforces strict execution environment invariants:
     - `GIT_NO_LAZY_FETCH=1`: Instructs Git not to attempt automatic on-demand remote object downloads in partial, blobless, or treeless clones.
     - `GIT_OPTIONAL_LOCKS=0`: Requests that Git omit acquiring optional locks (such as updating file stat caches in `.git/index` during read-only inspection operations).
     - `GIT_TERMINAL_PROMPT=0`: Disables interactive credential and passphrase prompts, guaranteeing immediate non-blocking failure instead of hanging indefinitely.
   - These environment variables are scoped strictly to the child process via `std::process::Command::env` and never modify the parent process environment, global user settings, or repository configuration.

3. **Date Constraints, Bounded Safety Cap, and Truncation Discoverability:**
   - Supported date constraints (`--since`, `--until`) are pushed directly down into the Git subprocess query, ensuring Git only traverses commits within the requested date boundaries.
   - Subprocess invocations query up to `DEFAULT_MAX_COMMITS_UNBOUNDED + 1` (1001 commits), establishing an upper safety cap against unbounded memory consumption on large repositories (e.g. 100,000+ commits).
   - If more than 1000 commits match, `TimelineView` marks `is_truncated = true`. This is exposed explicitly in machine-readable JSON envelopes (`"is_truncated": true`) and via human-readable CLI warning banners. DevContext never silently presents a capped history as a complete history.
   - The user-specified `--limit` is applied as the final global limit only after merging and chronologically sorting all matching Git and DevContext records.
   - Shallow clones (e.g. `git clone --depth 1`) are fully supported; commits are queried up to the shallow boundary without triggering network fetches or failing on grafted commit roots.

4. **Structured Delimitation and Robust Parsing:**
   - Commits are extracted using non-printable ASCII control characters: record separator (`%x1e`, byte 0x1E) between commits, and unit separator (`%x1f`, byte 0x1F) between fields (`%H`, `%an`, `%ae`, `%aI`, `%B`).
   - This guarantees that commit messages containing arbitrary whitespace, multiple lines, single/double quotes, commas, square brackets, and UTF-8 emojis (e.g., 🚀, ⚡) parse deterministically without string corruption.
   - Commits with empty messages gracefully fall back to displaying the short commit hash, and atypical authors (e.g., missing email or name) are formatted safely without empty brackets.

5. **Ephemeral Timeline Assembly (No Commit Duplication):**
   - DevContext will **not** store or sync full Git commit histories into the SQLite database.
   - The timeline is assembled dynamically in memory: DevContext records are queried from SQLite, Git commits are read from `git log`, merged into a unified `TimelineEvent` stream, sorted chronologically by UTC timestamp, and paginated.
   - Date range filtering (`--since`, `--until`) predictably handles both full RFC-3339 timestamps and date-only `YYYY-MM-DD` strings (with `--until` covering the full day up to 23:59:59.999Z).

## Technical Guarantees vs. Limitations

### Guarantees Provided by DevContext's Design
- **Command Whitelist:** DevContext's source code contains no logic to execute mutating Git subcommands or modify Git configuration.
- **Offline Invariant:** Invocations pass `GIT_NO_LAZY_FETCH=1` and `GIT_TERMINAL_PROMPT=0` to prevent partial-clone network fetching and interactive terminal hangs.
- **Environment Non-Interference:** DevContext never alters the developer's global environment variables or repository `.git/config`.
- **Fault-Tolerant Fallback:** When Git is absent or fails to launch, DevContext continues functioning completely for all non-Git project memory features without crashing.
- **Transparent Truncation:** When Git commit retrieval hits the 1000-commit safety cap, `is_truncated` is visibly surfaced to the user.

### Subprocess Execution Realities and Limitations
- **Process Delegation:** DevContext delegates metadata retrieval to the system `git` binary found on `$PATH`. It is not an OS-level sandbox or microkernel.
- **`GIT_OPTIONAL_LOCKS=0` Scope:** This setting asks Git to skip optional locking mechanisms (like index stat cache updates during reads). It is not an operating-system-level filesystem write block; safety from repository mutation is enforced by only invoking read-only Git subcommands.
- **Custom Wrappers and Aliases:** If a user or environment has replaced `git` on `$PATH` with a custom script, hook, or wrapper that performs unintended operations or produces non-standard output, DevContext is bound by the behavior and exit status of that external binary.

## Consequences

### Positive
- Zero external crate dependencies added to `Cargo.toml`.
- Fast compilation and minimal binary size.
- Safe by default: no Git mutation, no network requests, no lock acquisition.
- No stale Git cache: timeline always reflects real-time repository state.
- Graceful degradation in non-Git workspaces.
- Truncation is always discoverable.

### Trade-offs
- Git timeline integration requires a working `git` binary installed on `$PATH`. In environments without Git, timeline displays only authored DevContext records.
