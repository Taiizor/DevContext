# DevContext Support & Community Guidelines

Welcome to the DevContext community! DevContext is an open-source, local-first project memory system.

---

## 1. Where to Get Help

- **Documentation:**
  - [Installation & Upgrade Guide](../docs/user/installation-and-upgrade.md)
  - [Troubleshooting & Diagnostics](../docs/user/troubleshooting.md)
  - [Backup & Disaster Recovery](../docs/user/backup-and-recovery.md)
  - [Privacy & Storage Model](../docs/user/privacy-and-storage.md)
  - [Sample Project & Workflows](../examples/sample-project/README.md)
- **Built-in Diagnostics:** Run `devcontext doctor` in your project directory for self-diagnosis of SQLite health and schema status.
- **GitHub Discussions / Issues:** Use GitHub Issues for reproducible bug reports, feature requests, and documentation feedback.

---

## 2. Submitting Bug Reports

To help maintainers diagnose and resolve issues effectively, please ensure your report includes:

1. **Exact Version & Client:** Output of `devcontext --version` and whether you are using the CLI or the VS Code extension.
2. **Environment:** OS version (Windows, Linux, macOS), architecture (x86_64 or aarch64), and VS Code version if applicable.
3. **Minimal Reproduction Steps:** Concise, reproducible sequence of commands starting from a fresh workspace (`devcontext init`).
4. **Sanitized Error Output:** Error logs or JSON output. **Always redact personal file paths, tokens, and sensitive project names**.

> [!CAUTION]
> **Privacy Warning:**
> Never attach credentials, passwords, private keys, authorization tokens, proprietary source code, or your raw `.devcontext/devcontext.db` database to public issues.
> For security vulnerabilities, refer to [SECURITY.md](../SECURITY.md) for private reporting channels.

---

## 3. How Feature Requests are Evaluated

DevContext is deliberately built around strict architectural principles. When proposing a new feature, keep in mind:

- **Local-First & Privacy Tenets:** Features that require mandatory external servers, accounts, or telemetry will not be accepted.
- **Human-Owned Knowledge:** Features that silently capture arbitrary source code, terminal input, or synthesize speculative rationale without developer consent are out of scope.
- **Non-Destructive Operations:** Features that manipulate user data must provide confirmation dialogs, dry-run previews, and automatic backups.

---

## 4. Triage and Response Expectations

- **Reproducible Bugs:** Prioritized based on severity (data integrity and crash issues receive immediate attention).
- **Duplicate Reports:** Maintainers will link and close duplicate issues to keep discussions consolidated.
- **Response Times:** DevContext is maintained by open-source contributors. While we strive to review issues promptly, **response times are not guaranteed**.
