# ADR-0006: Audit Log Written in Same Database Transaction

## Status

Accepted. Non-Negotiable §10.

## Context

A separate-process audit pipeline risks losing events on crash. A
best-effort audit-log is not compliant.

## Decision

Every entity write is preceded by an `INSERT INTO lcc_audit.events (...)`
in the **same transaction**. The audit table is INSERT-only for the
`lcc_audit_writer` role; UPDATE/DELETE are revoked.

A separate `audit-integrity-worker` verifies the checksum chain nightly.

## Consequences

- Atomic entity + audit writes.
- The audit_log role enforces the immutability invariant at the DB level.
- Performance: 1 INSERT per business write (indexed by `member_id`,
  `resource_type+resource_id`, `occurred_at`).
