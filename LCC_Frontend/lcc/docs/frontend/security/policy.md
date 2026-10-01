# Security policy

The frontend obeys the following non-negotiables:

1. No direct LinkedIn API calls.
2. No business state in client-side storage (cookies/localStorage/IndexedDB).
3. No PII, JWTs, or refresh tokens in client logs.
4. Tier-2+ actions require `<ApprovalDialog>` confirmation.
5. KB citations mandatory for Tier-3+.
6. Tier-5 actions require typed confirmation ("Apply").

## Headers

- CSP via `next.config.mjs` (script/style nonces).
- HSTS 1y preload.
- `X-Content-Type-Options: nosniff`.
- `Referrer-Policy: strict-origin-when-cross-origin`.
- `Permissions-Policy: camera=(), microphone=(), geolocation=()`.

## Logging

- All client-side logs scrubbed for emails, JWTs, bearer tokens via `@lcc/test-utils/mock-logger`.
- The extension's `src/lib/redaction.ts` is the canonical implementation.

## Reporting

- Email: `security@okeson.example`.
- PGP key in `SECURITY.md`.
