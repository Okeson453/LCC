# ADR-0003: Integration Gateway as Sole LinkedIn Credential Holder

## Status

Accepted.

## Context

LinkedIn API tokens are sensitive. Spreading them across services increases
the risk of accidental disclosure and complicates rotation.

## Decision

Only the `integration-gateway` service holds LinkedIn OAuth credentials.
Every other service communicates with LinkedIn **through** it, by
presenting a verified permit-token.

## Consequences

- Single rotation surface for LinkedIn credentials.
- Single blast-radius for upstream LinkedIn failures.
- Easier to enforce mTLS to LinkedIn from a single egress point.
