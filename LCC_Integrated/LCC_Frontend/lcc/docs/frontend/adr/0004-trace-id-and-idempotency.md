# ADR 0004: trace_id + Idempotency-Key for every API mutation

## Status

Accepted (binding).

## Decision

- Every client-initiated mutation sets:
  - `x-trace-id: <uuid>` — used to correlate logs and audit events.
  - `Idempotency-Key: <uuid>` — used by the gateway to coalesce retries.
- Reads may omit the idempotency key (idempotency is implicit on GET).

## Consequences

- Required in tests: any mutation that fails to set both headers fails the contract suite.
- Server-side enforcement also added; the gateway will reject mutations without
  these headers.
