# ADR 0003: No direct LinkedIn APIs from the frontend

## Status

Accepted (binding).

## Context

The frontend must not have credentials to LinkedIn API, must not hold refresh
tokens, and must not implement LinkedIn-side workflows internally.

## Decision

- All LinkedIn reads/writes proxy through the gateway service.
- The browser extension shows a Tier-1 read overlay only — no DOM mutations.
- CI fails if any code outside the gateway service imports `linkedin-insights` or
  other private APIs.

## Consequences

- Every browser extension feature must derive its information from the gateway
  WS/SSE/HTTP streams.
- Tier-2 writes always require the user to land in the dashboard and resolve a
  `<ApprovalDialog>` before the action is dispatched.
