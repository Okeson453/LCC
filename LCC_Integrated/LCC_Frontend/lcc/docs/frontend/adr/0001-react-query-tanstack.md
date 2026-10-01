# ADR 0001: TanStack Query as the single data-layer cache

## Status

Accepted.

## Context

The frontend needs a deterministic, server-state cache that participates in
trace propagation, retries, and synchronous invalidation when realtime events
arrive. The candidate options were:

- Redux Toolkit Query
- SWR
- TanStack Query (React Query)

## Decision

TanStack Query.

## Consequences

- Mature ecosystem, TS-first.
- First-class DevTools.
- Clean integration with `@lcc/realtime` via `useRealtimeInvalidator`.
- Easy replacement if the team wants a different solution — given the API layer
  is fully isolated under `lib/api`, the impact of swap is local.
