# ADR 0003: Default storage location and Git exclusion strategy

- **Status:** Accepted
- **Date:** 2026-10-30

## Context

DevContext records developer-authored notes, technical decisions, and local context for a project workspace. Some notes or decisions may contain private thoughts, draft ideas, or proprietary internal context that the developer does not intend to share publicly with git remotes.

If DevContext were to write directly to shared, tracked Git files without user knowledge, or if it were to mutate the shared `.gitignore` file silently, it would create repository noise, unexpected git diffs, and risk accidental commits.

## Decision

1. **Default Storage Path:**
   - DevContext stores local project data in a hidden folder at the project/repository root: `.devcontext/`.
   - The primary SQLite database resides at `.devcontext/devcontext.db`.
   - Local attachments or temporary files reside at `.devcontext/attachments/`.

2. **Git Exclusion Strategy:**
   - DevContext will **never automatically edit the tracked `.gitignore` file**. Mutating tracked files can cause unrequested merge conflicts or unwanted commits in collaborative repositories.
   - For Git repositories, DevContext uses Git's repository-local exclude file: `.git/info/exclude`.
   - Upon project initialization (`devcontext init`), DevContext inspects `.git/info/exclude`. If `.devcontext/` is not present, it appends `.devcontext/` with an explanatory comment, ensuring zero git status pollution without dirtying tracked files.
   - For users who explicitly want to share selected DevContext records (such as ADRs) with their team, DevContext provides an export command to generate clean Markdown files in an explicit, user-designated documentation folder (e.g., `docs/decisions/`).

## Consequences

### Positive
- Zero git noise: `git status` remains clean immediately after initialization.
- Safe by default: Private notes are never accidentally committed or pushed to remote repositories.
- Transparent: Does not alter public repository files tracked by team members.

### Trade-offs
- Local excludes via `.git/info/exclude` are per-clone. When a developer re-clones the repository, `.devcontext/` won't be excluded until `devcontext init` is run in the new clone or manual ignore configuration is added.
