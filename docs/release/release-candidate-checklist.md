# Release Candidate Verification Checklist (v0.1.0-rc1)

This checklist verifies release candidate readiness across engineering, compatibility, security, accessibility, and documentation criteria.

---

## 1. Build and Binary Integrity
- [x] Rust workspace builds clean in release profile (`cargo build --release`).
- [x] Release CLI executable is standalone without dynamic runtime dependencies (`devcontext.exe` ~4.05 MB).
- [x] VS Code extension bundles cleanly (`bun run build`).
- [x] VSIX package generation succeeds without dependency errors (`bun run package`).
- [x] Version numbers are consistent across `Cargo.toml`, CLI `--version`, and extension `package.json` (`0.1.0`).

## 2. Automated Test Verification
- [x] Rust unit and integration test suite passes (`cargo test --workspace`: 103 passed, 0 failed).
- [x] Rust code formatting passes (`cargo fmt --check`).
- [x] Rust Clippy static analysis passes with zero warnings (`cargo clippy --all-targets --all-features -- -D warnings`).
- [x] TypeScript type-check passes without errors (`bun run typecheck`).
- [x] TypeScript test suite passes (`bun test`: 68 passed, 0 failed).
- [x] Real VS Code Extension Host integration tests pass.
- [x] Cross-platform GitHub Actions CI passes on Ubuntu, Windows, and macOS (Run ID `41129383061`).

## 3. Edge Cases and Filesystem Robustness
- [x] Workspaces with spaces in path name pass (`test_workspace_with_spaces_in_path`).
- [x] Non-ASCII, Unicode, and Turkish characters in paths and content pass (`test_workspace_with_unicode_and_turkish_path`).
- [x] Standalone folders without Git repositories function cleanly (`test_non_git_workspace_lifecycle`).
- [x] Multi-root workspace projects maintain strict isolation (`test_multi_root_workspace_strict_isolation`).
- [x] Corrupted database headers and invalid files are safely reported (`test_database_corruption_detection_and_safe_reporting`).
- [x] Schema migrations are transactional and idempotent (`test_schema_migration_idempotence_and_preservation`).

## 4. Performance Targets
- [x] Common CLI startup is under 200 ms (observed: ~13.7 ms average for `--help`, ~13.3 ms for `status --json`).
- [x] FTS5 search latency on 10,000 records is under 300 ms (observed: ~102.1 ms average in unoptimized debug mode).
- [x] Bounded Git history scanning prevents memory exhaustion on large repositories.
- [x] Extension activation avoids expensive disk scans (`onStartupFinished`).

## 5. Security and Privacy
- [x] Zero telemetry and zero network calls verified in codebase audit.
- [x] Machine-specific absolute paths are relativized or redacted during export.
- [x] Zip Slip path traversal, ADS colon streams, and null byte injection attacks are rejected.
- [x] Saved commands are strictly stored as metadata and never executed automatically.
- [x] Destructive operations (import, delete-data) enforce mandatory pre-operation backups.
- [x] Emergency safety snapshots are preserved on disk during restore failure.

## 6. Accessibility and UI Standards
- [x] All tree view items provide explicit screen-reader `accessibilityInformation` labels.
- [x] Theme colors use standard VS Code `ThemeColor` and `ThemeIcon` tokens for contrast compatibility across Dark+, Light+, and High Contrast themes.
- [x] Keyboard shortcuts configured for primary workflows (`Ctrl+Alt+D S`, `Ctrl+Alt+D N`, `Ctrl+Alt+D A`).
- [x] Modal confirmations provide clear explanation and actionable buttons.

## 7. Documentation
- [x] `README.md` updated with release status and quick start.
- [x] `CHANGELOG.md` updated with complete Phase 0-8 change log.
- [x] `docs/user/troubleshooting.md` created.
- [x] `docs/user/installation-and-upgrade.md` created.
- [x] `docs/user/backup-and-recovery.md` created.
- [x] `docs/user/privacy-and-storage.md` created.
- [x] `docs/release/marketplace-listing.md` and `screenshots.md` prepared.
- [x] `PROGRESS.md` updated with verification evidence.
