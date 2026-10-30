# ADR 0002: Human-owned knowledge and consent-based capture

-   **Status:** Accepted as initial direction
-   **Date:** 2026-10-30

## Context

A project memory tool can easily become noisy or misleading if it
automatically turns file changes, commit messages, or inferred patterns
into claims about developer intent.

## Decision

-   Notes and technical decisions are authored by the developer.
-   Git metadata may be collected as metadata, but it must be labeled as
    such.
-   Automated suggestions must be clearly labeled and require user
    confirmation before becoming project knowledge.
-   The MVP does not automatically capture terminal input, environment
    variable values, arbitrary source contents, or secrets.

## Consequences

Positive: - The stored knowledge is more trustworthy. - Privacy
expectations are easier to explain. - The tool avoids becoming a
surveillance or activity-recording product.

Trade-offs: - Users must explicitly record important rationale. -
Automated convenience is limited in the first release.

## Notes

Any future automatic collector must document what it reads, what it
stores, how it can be disabled, and how collected data can be inspected
and deleted.
