# Architecture Overview

## Two-Engine Split

OKESON-LCC is split into two engines that communicate only via gRPC
contracts (`proto/`) and the event bus:

### Rust Core Engine (`engine/core/`)
- **Transactional & safety-critical.**
- Owns the database, RLS, audit-log (in-tx), idempotency, circuit breakers,
  permit-token verification, all LinkedIn credentials.
- 13 services + 6 workers + 3 CLI binaries.

### Python Intelligence Engine (`engine/intelligence/`)
- **LLM orchestration, ML scoring, RAG, voice.**
- 5 services + 3 background workers.
- Side-effects: KB records, voice samples, model artifacts.
- **No direct database writes for member-scoped actions** — only the
  `kb-client` / `vector-db` / `event-bus` packages touch shared stores,
  and only through audited APIs.

### Boundary rules
Enforced by `infra/ci/scripts/check_boundaries.sh` (CI-gated):
- Rust Core never imports from Python Intelligence (no path-based reference).
- Python Intelligence never imports from Rust Core (no path-based reference).
- The `lcc-integrations` crate may only be used by `integration-gateway`.

## Single Compliance Governor

The Compliance Governor is the **only** entity that issues permits for
external actions. Every external write-side request:
1. Hits the relevant service (e.g., `outreach-svc`).
2. The service calls `compliance-governor /evaluate` with `member_id + action_type + target_kind + context`.
3. The governor runs 8 sequential guards:
   `daily_cap → cooldown → duplicate_target → account_health → grounding → restriction_flag → approval_state → session_pacing`
4. If `allow`, returns a JWT permit-token (≤60s TTL, audience="integration-gateway").
5. The service then calls `integration-gateway /execute` with the permit-token.
6. The integration-gateway verifies the JWT, checks idempotency, runs the
   circuit breaker, and dispatches via Track A (REST) or Track B (WSS).

## Audit in same TX

Every entity write is accompanied by an `audit_log` row inserted in the
**same DB transaction**. The audit table is INSERT-only for the
`lcc_audit_writer` role; no UPDATE, no DELETE.

## Data flow

```
                ┌─────────────────┐
   client  ───▶ │  api-gateway    │ ─── proxy routes
                └────────┬────────┘
                         ▼
            ┌──────── domain svc (outreach-svc, content-svc, ...)
            │           │
            │           ├──▶ compliance-governor   (8 guards, ≤200ms p99)
            │           │      │
            │           │      └─▶ permit-token JWT
            │           │
            │           ▼
            │      integration-gateway (verify JWT → idem → circuit-breaker)
            │           │
            │           ├──▶ Track A  (REST to LinkedIn via credentials)
            │           └──▶ Track B  (WSS to browser extension; human-in-loop)
            │
            └──▶ audit-svc (in-tx audit_log insert)
```

## Concurrency & Idempotency

- Every write-side Integration call carries `idempotency_key = "{action_type}:{resource_id}:{version}"`.
- The integration-gateway stores the key in Redis (7d TTL) and Postgres.
- Replays return the original response.

## Storage

- **Postgres** — primary OLTP store with RLS per member.
- **Redis** — idempotency cache, circuit-breaker state, event bus (Streams).
- **Qdrant** — vector store for KB, voice samples, opportunity signals.
- **Materialized views** in Postgres for analytics aggregation.
