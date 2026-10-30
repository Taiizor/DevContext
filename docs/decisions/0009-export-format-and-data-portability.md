# ADR 0009: Export and import format specification and schema versioning

- **Status:** Accepted
- **Date:** 2026-10-30

## Context

A fundamental product tenet of DevContext is user ownership and data portability. If a developer ceases using DevContext or migrates between environments, their technical knowledge must not be trapped in a proprietary or undocumented binary format.

Furthermore, importing external or transferred context into an existing workspace carries risks of data corruption, path leaks, or ID collisions.

## Decision

1. **Standard Export Formats:**
   DevContext supports three complementary export targets:
   - **Markdown Export:** A standalone folder containing human-readable Markdown files organized by record type (e.g. `decisions/ADR-0001.md`, `notes/debugging-notes.md`). Includes standard YAML frontmatter for metadata.
   - **Versioned JSON (`export-v1.json`):** A single machine-readable document strictly validated against `schemas/export-v1.schema.json`. Contains complete record graphs, references, and project identity.
   - **Portable Archive (`.devcontext-bundle.zip`):** A standard zip file containing the root `export-v1.json` alongside Markdown documents and referenced attachments.

2. **Privacy & Path Sanitization:**
   - Absolute filesystem paths (e.g. `C:/Users/username/repos/project`) are sanitized and converted into workspace-relative paths (`./src/main.rs`).
   - If an external file target cannot be relativized, its machine-specific prefix is redacted to prevent accidental exposure of personal usernames or directory structures.

3. **Import Safety Protocol:**
   - **Validation:** Imported JSON is strictly validated against the export schema version before parsing.
   - **Pre-Import Backup:** Before any import operation that modifies the local database, DevContext generates an automatic backup snapshot in `.devcontext/backups/`.
   - **Conflict Strategy:** If incoming record UUIDs collide with existing local records:
     - Default policy is `prompt` in interactive mode.
     - In non-interactive mode, `--on-conflict <skip|overwrite|generate-new-id>` must be explicitly passed.
   - **Path Traversal Protection:** Archive extraction strictly checks for path traversal vulnerabilities (e.g. `../` or absolute destination paths) and rejects any hazardous entries.

## Consequences

### Positive
- True data portability: Markdown files can be read in any editor, hosted in git, or viewed offline.
- Full backup safety: Destructive imports cannot accidentally wipe uncommitted local knowledge.
- Stable, documented interchange schema.

### Trade-offs
- Maintaining two parallel export representations (Markdown + JSON) requires bidirectional mapping logic in the export engine.
