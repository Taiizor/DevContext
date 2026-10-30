# DevContext --- Implementation Progress

> This file is the live execution tracker. Update it whenever work is
> started, completed, blocked, or materially changed.
>
> **Rule:** A checkbox may be checked only when the implementation
> exists and the stated verification evidence has been recorded.

## Project status

-   **Current phase:** Phase 0 --- Discovery and product contract
-   **Overall status:** Not started
-   **Last updated:** 2026-10-30
-   **Current milestone:** Product definition
-   **Current blocker:** None recorded

### Status legend

-   `[ ]` Not started
-   `[/]` In progress
-   `[x]` Complete and verified
-   `[!]` Blocked
-   `[-]` Deferred / intentionally out of scope

## Phase summary

  Phase                                       Status        Completion date   Notes
  ------------------------------------------- ------------- ----------------- -------
  0\. Discovery and product contract          Not started   ---               
  1\. Repository and engineering foundation   Not started   ---               
  2\. Core domain and storage                 Not started   ---               
  3\. CLI MVP                                 Not started   ---               
  4\. Git integration and timeline            Not started   ---               
  5\. Search and references                   Not started   ---               
  6\. VS Code extension MVP                   Not started   ---               
  7\. Export/import and data safety           Not started   ---               
  8\. Quality and release readiness           Not started   ---               
  9\. Public MVP release                      Not started   ---               

------------------------------------------------------------------------

## Active work

> Keep this section short. Move finished work to the completed log.

-   No active tasks.

## Blockers and risks

  ID    Blocker / risk   Impact   Owner   Next action   Status
  ----- ---------------- -------- ------- ------------- --------
  ---   None recorded    ---      ---     ---           ---

## Phase 0 --- Discovery and product contract

-   [ ] Confirm product name, repository name, license, and supported
    platforms.
-   [ ] Define MVP and explicitly reject non-MVP features.
-   [ ] Write user stories for initialization, notes, decisions, search,
    timeline, and export.
-   [ ] Define privacy policy and collection boundaries.
-   [ ] Decide default storage location and Git ignore behavior.
-   [ ] Decide multi-root workspace behavior.
-   [ ] Define stable project identity and clone detection behavior.
-   [ ] Choose Rust/TypeScript workspace strategy.
-   [ ] Define CLI-to-core and extension-to-core communication.
-   [ ] Draft initial data model.
-   [ ] Draft export schema version 1.
-   [ ] Record major architecture decisions in `docs/decisions/`.

**Verification evidence:** - Pending.

## Phase 1 --- Repository and engineering foundation

-   [ ] Create repository structure.
-   [ ] Add root `AGENTS.md` and contribution guidance.
-   [ ] Add license, README, SECURITY, CODE_OF_CONDUCT, and CONTRIBUTING
    documents.
-   [ ] Configure formatting and linting for Rust and TypeScript.
-   [ ] Configure CI for Windows, Linux, and macOS.
-   [ ] Add dependency update policy.
-   [ ] Add documented commit convention.
-   [ ] Add issue and pull request templates.
-   [ ] Add build scripts and developer setup instructions.
-   [ ] Add test fixtures and sample repositories.
-   [ ] Add release versioning strategy.
-   [ ] Ensure no secrets or local database files are committed.

**Verification evidence:** - Pending.

## Phase 2 --- Core domain and storage

-   [ ] Define domain types and validation rules.
-   [ ] Implement project initialization and stable identity.
-   [ ] Implement SQLite connection management.
-   [ ] Implement schema migrations.
-   [ ] Implement transactional repository interfaces.
-   [ ] Implement note CRUD.
-   [ ] Implement decision CRUD.
-   [ ] Implement milestone and log entry persistence.
-   [ ] Implement tags and references.
-   [ ] Implement saved command records (storage only).
-   [ ] Implement data integrity checks.
-   [ ] Implement database backup and restore primitives.
-   [ ] Add migration tests.
-   [ ] Add corruption and locked-database handling.
-   [ ] Add concurrent access and interrupted-write tests.

**Verification evidence:** - Pending.

## Phase 3 --- CLI MVP

-   [ ] `devcontext init`
-   [ ] `devcontext status`
-   [ ] `devcontext note add`
-   [ ] `devcontext note list`
-   [ ] `devcontext note show`
-   [ ] `devcontext note edit`
-   [ ] `devcontext note delete`
-   [ ] `devcontext decision add`
-   [ ] `devcontext decision list`
-   [ ] `devcontext decision show`
-   [ ] `devcontext decision update`
-   [ ] `devcontext milestone add`
-   [ ] `devcontext log add`
-   [ ] `devcontext search`
-   [ ] `devcontext timeline`
-   [ ] `devcontext export`
-   [ ] `devcontext import`
-   [ ] `devcontext doctor`
-   [ ] CLI integration tests and command documentation.

**Verification evidence:** - Pending.

## Phase 4 --- Git integration and timeline

-   [ ] Detect Git repository root.
-   [ ] Read current branch and HEAD.
-   [ ] Read bounded commit metadata.
-   [ ] Support configurable history depth.
-   [ ] Link records to commits manually.
-   [ ] Link current HEAD to a record.
-   [ ] Assemble combined timeline.
-   [ ] Distinguish Git events from authored records.
-   [ ] Handle shallow clones, detached HEAD, no Git, and empty
    repositories.
-   [ ] Keep Git integration read-only.
-   [ ] Add Git fixture tests.

**Verification evidence:** - Pending.

## Phase 5 --- Search and references

-   [ ] Implement full-text indexing.
-   [ ] Index titles, bodies, tags, and selected metadata.
-   [ ] Implement snippets and ranking.
-   [ ] Add record type, status, tag, and date filters.
-   [ ] Implement file and folder references.
-   [ ] Implement commit and branch references.
-   [ ] Implement URL references.
-   [ ] Implement command references.
-   [ ] Implement unavailable-target states.
-   [ ] Add reindex command.
-   [ ] Add search performance tests.
-   [ ] Document search behavior.

**Verification evidence:** - Pending.

## Phase 6 --- VS Code extension MVP

-   [ ] Create extension manifest and activation strategy.
-   [ ] Detect workspace roots and multi-root workspaces.
-   [ ] Locate or provision compatible core executable.
-   [ ] Implement extension/core version handshake.
-   [ ] Add project initialization command.
-   [ ] Add activity bar container and project overview.
-   [ ] Add notes and decisions tree views.
-   [ ] Add timeline view.
-   [ ] Add search command and quick-pick results.
-   [ ] Add note creation/editing UI.
-   [ ] Add decision creation/editing UI.
-   [ ] Add milestone and log commands.
-   [ ] Add file and selection reference actions.
-   [ ] Add reference-opening actions.
-   [ ] Add export/import preview and confirmation.
-   [ ] Add privacy and storage settings.
-   [ ] Handle missing/incompatible core executable.
-   [ ] Add extension host tests and manual test checklist.
-   [ ] Ensure long operations do not block the UI.

**Verification evidence:** - Pending.

## Phase 7 --- Export/import and data safety

-   [ ] Finalize versioned JSON schema.
-   [ ] Implement Markdown export.
-   [ ] Implement portable archive export.
-   [ ] Add export preview.
-   [ ] Add path redaction.
-   [ ] Add import preview and conflict resolution.
-   [ ] Add import schema upgrades.
-   [ ] Back up before destructive imports.
-   [ ] Add data deletion command and UI.
-   [ ] Add data inspection command.
-   [ ] Add round-trip tests.
-   [ ] Test malformed and large archives.
-   [ ] Document compatibility guarantees.

**Verification evidence:** - Pending.

## Phase 8 --- Quality and release readiness

-   [ ] Test Windows, Linux, and macOS.
-   [ ] Test supported VS Code versions.
-   [ ] Test spaces and non-ASCII paths.
-   [ ] Test read-only workspaces.
-   [ ] Test network drives where supported.
-   [ ] Test large repositories.
-   [ ] Test no-Git and multi-root workspaces.
-   [ ] Test database lock and recovery.
-   [ ] Test upgrade paths.
-   [ ] Audit logs for sensitive data.
-   [ ] Audit file and process operations.
-   [ ] Review keyboard navigation and accessibility.
-   [ ] Review theme compatibility and contrast.
-   [ ] Measure startup and command latency.
-   [ ] Write troubleshooting guide.
-   [ ] Prepare changelog and release notes.
-   [ ] Prepare Marketplace listing and screenshots.
-   [ ] Complete release-candidate feedback cycle.

**Verification evidence:** - Pending.

## Phase 9 --- Public MVP release

-   [ ] Publish source repository.
-   [ ] Publish VS Code extension.
-   [ ] Publish CLI binaries.
-   [ ] Publish documentation and examples.
-   [ ] Add sample project data.
-   [ ] Add issue templates and support guidelines.
-   [ ] Publish release announcement/demo.
-   [ ] Triage early feedback.
-   [ ] Prioritize reproducible bugs.

**Verification evidence:** - Pending.

------------------------------------------------------------------------

## Completed work log

Add newest entries first.

  Date   Phase / task   Change                             Verification evidence
  ------ -------------- ---------------------------------- -----------------------
  ---    ---            No implementation completed yet.   ---

## Deferred decisions

  -----------------------------------------------------------------------
  Topic             Decision needed   Deadline /        Status
                                      trigger           
  ----------------- ----------------- ----------------- -----------------
  License           Choose a license  Before public     Open
                    compatible with   repository        
                    intended          release           
                    community and                       
                    distribution                        
                    goals.                              

  Global storage    Decide whether to Before storage    Open
                    support a global  implementation    
                    data directory in                   
                    MVP.                                

  Core distribution Decide packaging  Before extension  Open
                    strategy for Rust MVP               
                    core binaries                       
                    used by the                         
                    extension.                          

  Shared knowledge  Decide exact UX   Before            Open
                    for selectively   export/import     
                    committing        release           
                    Markdown records.                   
  -----------------------------------------------------------------------

## Update protocol for agents

After every meaningful work session: 1. Update the current phase and
overall status. 2. Mark only verified tasks as complete. 3. Add exact
test commands and outcomes to verification evidence. 4. Record changed
files or relevant commit hashes in the completed work log. 5. Add
blockers with a concrete next action. 6. Update the phase summary table.
7. If scope or architecture changed, update `ROADMAP.md` and
create/update an ADR. 8. Never mark a task complete merely because code
was written; run the relevant checks.
