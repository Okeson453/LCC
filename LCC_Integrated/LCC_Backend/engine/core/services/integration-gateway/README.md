# integration-gateway

The **only service** permitted to call LinkedIn-facing endpoints (Non-Negotiable §1).

## Responsibilities

1. Verify `permit_token` from the Compliance Governor before any external action.
2. Route to Track A (official API) or Track B (browser-assist WSS).
3. Enforce idempotency (Redis + Postgres-backed 7-day TTL).
4. Run circuit breaker (CLOSED → OPEN → HALF_OPEN).
5. Detect restriction signals → account-wide pause (axiom 6).
6. Emit `audit.event` for every executed action.
7. Hold the only Vault path to LinkedIn OAuth tokens.

## Endpoints

- `POST /internal/integration/execute` — execute via Track A (org-page post).
- `POST /internal/integration/execute-track-b` — execute via Track B (browser-assist).
- `WSS  /track-b` (port 8443) — browser-assist extension messaging.
- `GET  /healthz`, `/readyz`, `/metrics`.

## SLOs

- Permit-token validation p99: **< 50ms**
- Available **24/7** (no scheduled downtime).

## Non-negotiables enforced

- §1: only this service holds LinkedIn credentials.
- §2: only this service verifies `permit_token`.
- §6: any ambiguous platform signal → account-wide pause.
- §13: every write-side call is idempotent.

## Local dev

```bash
just bootstrap
cargo run -p integration-gateway
```
