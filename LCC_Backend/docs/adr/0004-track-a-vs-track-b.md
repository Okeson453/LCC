# ADR-0004: Track A (REST) vs Track B (Browser Extension WSS)

## Status

Accepted.

## Context

LinkedIn's API rate-limits some actions (notably personal-profile PostPublish,
connection requests, DMs) such that a server-driven REST approach can cause
account restrictions. The browser-extension approach (human-in-the-loop, with
the user clicking the LinkedIn UI) avoids these limits.

## Decision

Two tracks:
- **Track A**: server-side REST, used for safe actions (comments, likes,
  profile views, follows, some company-page APIs).
- **Track B**: browser-extension oneshot, used for high-risk actions
  (personal-profile PostPublish, connection requests, DMs, sequence
  step sends, job application submissions, client proposal sends,
  executive outreach).

Track routing is decided by the integration-gateway based on
`action_type` (per spec §44).

## Consequences

- Track B requires the member to have the browser extension installed.
- Track B adds a delay (human confirmation) but is safer.
- All Track B actions are still evaluated by the Compliance Governor.
