# Security & Privacy Policy

DevContext is built around a local-first, privacy-by-default architecture. We take the security and integrity of your development environment and project context seriously.

---

## 1. Core Security Guarantees

- **No Remote Telemetry:** DevContext does not transmit telemetry, analytics, or repository content over the internet.
- **Zero Involuntary Capture:** DevContext does not sniff terminal activity, process memory, or environment variables.
- **Inert Saved Commands:** Commands are saved as static text and are never automatically executed.
- **Path Traversal Protection:** Archive extraction and file exports strictly validate relative paths and reject hazardous destinations (e.g. `../`).
- **SQL Injection Prevention:** All SQLite persistence operations utilize strictly parameterized queries.

---

## 2. Reporting a Vulnerability

If you discover a potential security vulnerability in DevContext (such as an unintentional secret exposure, arbitrary code execution, path traversal, or database corruption bug), please report it responsibly:

- **Do NOT open a public GitHub issue.**
- Submit your findings via GitHub Private Vulnerability Reporting or email the maintainers directly.
- Include detailed reproduction steps, operating system details, and a minimal reproducible test case.
- Maintainers will acknowledge reports within 48 hours and work with you on a coordinated disclosure timeline.

---

## 3. Supported Versions

| Version | Supported |
|---|---|
| `0.1.x` (Development / MVP) | Yes |
