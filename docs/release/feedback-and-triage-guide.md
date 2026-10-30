# DevContext Early Feedback and Issue Triage Guide

This guide defines the feedback collection, classification, and issue triage process for **DevContext**.

---

## 1. Zero-Telemetry Feedback Philosophy

DevContext enforces strict privacy invariants:
- **No Involuntary Tracking:** The application contains **zero background telemetry, analytics, or automated error reporting**.
- **User-Initiated Feedback:** All feedback, bug reports, and usability observations arrive through explicitly chosen developer channels:
  - GitHub Issues
  - GitHub Discussions
  - Security Vulnerability Reporting (private)
- **Manual Friction Tracking:** Maintainers track usability friction through qualitative review of reported workflows, reproducible bug reports, and explicit developer discussions.

---

## 2. Issue Classification Taxonomy

Issues are classified using a clear label taxonomy:

### A. Component Labels
- `component:core` — Rust domain logic, SQLite storage, migrations, or backup/restore.
- `component:cli` — Command line interface, arguments, output envelopes, exit codes.
- `component:vscode` — VS Code extension UI, tree views, commands, IPC, or binary resolution.
- `component:git` — Git adapter, commit parsing, timeline merging, or repository root detection.
- `component:export` — Export, import, schema upgrades, zip archives, or sanitization.
- `component:docs` — Documentation, guides, examples, or changelog.

### B. Severity & Impact Labels
- `severity:critical` — Data loss, database corruption, security vulnerability, or unrecoverable crash during standard operation.
- `severity:high` — Major feature broken with no workaround (e.g. search completely failing, import failure).
- `severity:medium` — Functional bug with a known workaround, or edge-case platform failure.
- `severity:low` — Cosmetic defect, minor UI alignment issue, or typo.

### C. Reproducibility Labels
- `status:reproducible` — Verified minimal reproduction steps confirmed by maintainers.
- `status:needs-repro` — Requires additional environment details or minimal reproduction case from reporter.
- `status:in-discussion` — Design decision or feature proposal being evaluated against product principles.

---

## 3. Triage Workflow

```text
[New Issue Opened]
       │
       ▼
[Privacy & Sanitization Check]
       │  Did user paste secrets, private paths, or raw DB?
       ├──► Yes: Edit/redact sensitive data immediately, notify reporter.
       ▼
[Security Vulnerability Check]
       │  Is this an undisclosed security flaw?
       ├──► Yes: Transfer to GitHub Private Vulnerability Reporting; close public issue.
       ▼
[Duplicate Check]
       │  Is this defect already reported?
       ├──► Yes: Label `duplicate`, link original issue, close.
       ▼
[Reproducibility Assessment]
       │  Can maintainers reproduce the issue on supported OS?
       ├──► No: Add `status:needs-repro` and request environment diagnostics.
       ▼
[Prioritization & Assignment]
       Assign `component:*` and `severity:*` labels.
       Schedule for milestone release based on severity.
```

---

## 4. Prioritization Matrix

Maintainers prioritize issues according to the following order:

1. **Data Integrity & Security:** Database corruption, backup restoration failure, or security boundary issues take precedence over all feature development.
2. **Platform & File System Regressions:** Failures in path resolution (spaces, Unicode, NTFS/ext4/APFS differences) or process execution.
3. **Core Workflow Deficiencies:** Bugs in note/decision persistence, search ranking accuracy, or timeline assembly.
4. **Usability Friction:** Repetitive manual actions that could be streamlined within existing product principles.
5. **Speculative Feature Requests:** Evaluated strictly against the local-first, zero-telemetry charter. Requests requiring cloud dependencies or mandatory accounts are politely closed.
