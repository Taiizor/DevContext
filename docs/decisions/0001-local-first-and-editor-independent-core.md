# ADR 0001: Local-first product with an editor-independent core

-   **Status:** Accepted as initial direction
-   **Date:** 2026-10-30

## Context

DevContext is intended to preserve project knowledge over long periods.
The information may include private technical notes, internal
architecture decisions, and command snippets. It should remain useful
without a hosted service and should not be tied permanently to one
editor.

## Decision

-   Core workflows operate locally and offline.
-   No account or remote service is required.
-   The core domain and storage logic are independent of VS Code.
-   VS Code is an initial client, alongside a standalone CLI.
-   Data is stored in a documented local format and can be exported.

## Consequences

Positive: - Users retain control over their project knowledge. - Core
logic can be reused by other clients. - The product can function in
restricted or offline environments. - Data portability is a first-class
requirement.

Trade-offs: - The project must maintain packaging and compatibility
across platforms. - Local backup and recovery become important product
responsibilities. - Cross-editor integration requires a stable client
protocol.

## Notes

This is an initial product direction. Changes require a new ADR rather
than silently changing the product's privacy or architecture
assumptions.
