# ADR-0005: Idempotency by Construction

## Status

Accepted.

## Context

Retries happen. Network partitions happen. Without idempotency, a retried
write could create duplicate posts, double-send DMs, or duplicate connection
requests.

## Decision

Every write-side Integration Layer call carries `idempotency_key = "{action_type}:{resource_id}:{version}"`.
The integration-gateway stores this key in:
- Redis (fast-path, 7d TTL)
- Postgres (durable path)

Replays within the TTL return the original response.

## Consequences

- Guaranteed at-most-once execution per (member, action, resource, version).
- Slight extra latency on the first call (Redis lookup).
- Audit-log entries are also idempotent by the same key.
