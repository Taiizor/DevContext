# Dependency Management Policy

- **Status:** Approved (Phase 1)
- **Target:** Rust workspace and Bun TypeScript workspace

---

## 1. Principles

1. **Minimal Dependencies:** DevContext values stability, fast builds, small binary sizes, and minimal attack surface. Never add a third-party dependency for trivial functionality that can be implemented cleanly with standard library utilities.
2. **License Compatibility:** All direct and transitive dependencies must have permissive, open-source licenses compatible with the dual **MIT / Apache-2.0** model (e.g. MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC). GPL, AGPL, or ambiguous proprietary licenses are strictly prohibited.
3. **No Hidden Telemetry:** Dependencies that include telemetry, analytics, or automated remote network requests are strictly forbidden.

---

## 2. Rust Dependency Rules

- All workspace members declare dependencies through the root `[workspace.dependencies]` table in `Cargo.toml` to guarantee consistent versions across crates.
- Lockfile (`Cargo.lock`) is committed to version control.
- Use crates with established maintenance records (e.g., `rusqlite`, `serde`, `serde_json`, `clap`, `thiserror`).
- Security audits: Periodically run `cargo audit` to scan for known advisory vulnerabilities.

---

## 3. TypeScript / Bun Dependency Rules

- **Bun is the sole package manager.** Never run `npm`, `pnpm`, or `yarn`.
- Lockfile (`bun.lock` or `bun.lockb`) is committed to version control.
- Avoid heavyweight runtime frameworks inside the VS Code extension. The extension should remain a thin consumer of the VS Code Extension API.
- Keep dependencies updated using `bun update`.

---

## 4. Vulnerability Response Timeline

- **Critical / High CVEs:** Addressed and patched within 7 days.
- **Medium / Low CVEs:** Addressed during standard release cycles or next milestone.
