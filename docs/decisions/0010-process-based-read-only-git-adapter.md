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

2. **Strict Read-Only and Offline Guarantees:**
   - The adapter will **only** invoke read-only querying commands:
     - `git rev-parse --is-inside-work-tree`
     - `git rev-parse --show-toplevel`
     - `git rev-parse --abbrev-ref HEAD` (branch name)
     - `git rev-parse HEAD` (head commit)
     - `git log --format=...` (bounded commit history)
     - `git cat-file -e <hash>` (commit existence check)
   - Mutating commands (`commit`, `branch`, `checkout`, `reset`, `stash`, `merge`, `config`, `hook`) are strictly prohibited.
   - Every subprocess invocation enforces strict execution environment invariants:
     - `GIT_NO_LAZY_FETCH=1`: Disables Git from performing lazy on-demand remote object downloads in partial or blobless/treeless clones.
     - `GIT_OPTIONAL_LOCKS=0`: Disables background operations and read queries from acquiring optional repository locks on `.git/index`.
     - `GIT_TERMINAL_PROMPT=0`: Disables interactive credential and passphrase prompts, guaranteeing immediate non-blocking failure instead of hanging indefinitely.
   - These environment variables are scoped strictly to the child process via `std::process::Command::env` and never modify the parent process environment, global user settings, or repository configuration.

3. **Bounded Memory and Shallow Clone Resilience:**
   - Subprocess invocations pass limit arguments directly to Git (`-n <limit>`), ensuring Git traverses only the required commit count and never loads unbounded commit histories into memory on large repositories (e.g. 100,000+ commits).
   - If an unbounded timeline query is requested, `TimelineService` enforces an upper safety retrieval cap (`DEFAULT_MAX_COMMITS_UNBOUNDED = 1000`).
   - Shallow clones (e.g. `git clone --depth 1`) are fully supported; commits are queried up to the shallow boundary without triggering network fetches or failing on grafted commit roots.

4. **Structured Delimitation and Robust Parsing:**
   - Commits are extracted using non-printable ASCII control characters: record separator (`%x1e`, byte 0x1E) between commits, and unit separator (`%x1f`, byte 0x1F) between fields (`%H`, `%an`, `%ae`, `%aI`, `%B`).
   - This guarantees that commit messages containing arbitrary whitespace, multiple lines, single/double quotes, commas, square brackets, and UTF-8 emojis (e.g., 🚀, ⚡) parse deterministically without string corruption.
   - Commits with empty messages gracefully fall back to displaying the short commit hash, and atypical authors (e.g., missing email or name) are formatted safely without empty brackets.

5. **Ephemeral Timeline Assembly (No Commit Duplication):**
   - DevContext will **not** store or sync full Git commit histories into the SQLite database.
   - The timeline is assembled dynamically in memory: DevContext records are queried from SQLite, Git commits are boundedly read from `git log`, merged into a unified `TimelineEvent` stream, sorted chronologically by UTC timestamp, and paginated.
   - Date range filtering (`--since`, `--until`) predictably handles both full RFC-3339 timestamps and date-only `YYYY-MM-DD` strings (with `--until` covering the full day up to 23:59:59.999Z).

## Technical Guarantees vs. Limitations

### Guarantees
- DevContext will never mutate Git working trees, branch references, index files, or repository configuration.
- DevContext will never perform network operations or trigger remote fetches via Git.
- DevContext will never acquire `.git/index` lock files during inspection.
- When Git is absent or broken, DevContext continues functioning completely for all non-Git project memory features without crashing.

### Limitations
- DevContext relies on process-level execution of the system `git` executable found on `$PATH`.
- In environments where Git is not installed, Git commit history and branch metadata are not available.
- If a user configures unusual system-level wrappers or custom scripts named `git` that do not conform to standard Git CLI semantics, behavior is limited by the external script's output.

## Consequences

### Positive
- Zero external crate dependencies added to `Cargo.toml`.
- Fast compilation and minimal binary size.
- Safe by default: no Git mutation, no network requests, no lock acquisition.
- No stale Git cache: timeline always reflects real-time repository state.
- Graceful degradation in non-Git workspaces.

### Trade-offs
- Git timeline integration requires a working `git` binary installed on `$PATH`. In environments without Git, timeline displays only authored DevContext records.
