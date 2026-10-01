# ADR-0002: Single Compliance Governor

## Status

Accepted.

## Context

Every external write-side action must be evaluated against a set of safety
rules (daily caps, cooldowns, account health, grounding, etc.). Distributing
this logic across services invites inconsistency.

## Decision

There is **one** Compliance Governor service. All write-side actions route
through it before any external dispatch. The Governor:
- Issues a JWT permit-token on allow.
- The integration-gateway verifies the JWT (HMAC, ≤60s TTL) before
  dispatching to LinkedIn.

## Consequences

- One place to add a new guard.
- One place to enforce 8 guards sequentially.
- Tight SLO: governor p99 < 200ms.
- Hot path for all outbound actions.
