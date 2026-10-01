# API reference

The TypeScript API surface lives in `packages/api-types` and is consumed via
`apps/web-dashboard/src/lib/api/*`. This document is the engineer-facing
reference.

## Errors

The `ApiErrorKind` enum classifies every error:

- `network`, `timeout`
- `unauthorized`, `forbidden`, `not_found`, `version_mismatch`
- `validation`, `rate_limited`
- `restricted` (member restricted)
- `governance_denied` (compliance blocker)
- `server_error`, `unknown`

## Auth

All requests include `Authorization: Bearer <jwt>` derived from the encrypted
session cookie.

## Trace & idempotency

- `x-trace-id: <uuid>`
- `Idempotency-Key: <uuid>` (POST/PATCH/PUT/DELETE)

## Retry policy

- 1 retry on `network` or `timeout`.
- No retry on `restricted`, `governance_denied`, `unauthorized`, `forbidden`.

## Pagination

List endpoints accept `?limit=50&cursor=...` and return
`{ items: T[], next_cursor?: string }`.
