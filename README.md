# OKESON-LCC — LinkedIn Command Center

A production-oriented LinkedIn manager platform: LLM-assisted content drafting,
compliance-governed outbound actions, opportunity discovery, and a single
multi-tenant database — with every external action gated behind a universal
approval flow.

This repository is the complete, post-audit monorepo: backend, frontend, and the
authoritative API contract, plus the full audit report and its remediation.

---

## What it does

- **Content & drafting** — LLM-assisted post and message drafts grounded in a
  knowledge base (RAG), with approval before anything leaves the system.
- **Compliance-governed outreach** — a single Compliance Governor issues
  short-lived permit tokens; the Integration Gateway (the only holder of
  LinkedIn credentials) refuses any action without one. Sequences cannot bypass
  the guards.
- **Opportunity discovery** — scoring services (ρ / ab_d / h_c) and the
  network CRM surface and rank leads and engagement opportunities.
- **Realtime dashboard** — WebSocket-first (SSE fallback, polling last resort)
  across five dashboard channels with 19 typed event schemas.
- **Three surfaces** — Next.js web dashboard (also a mobile PWA) and a
  Manifest V3 browser extension, sharing eight packages.

## Architecture

Two engines, one contract:

```
                    ┌────────────────────────────┐
  Frontend ────────▶│  Integration Gateway       │──▶ LinkedIn (sole credential holder)
  (Next.js + MV3)   │  + API Gateway             │
        ▲           └────────────┬───────────────┘
        │ WS/SSE                  │ gRPC / event bus
        │                         ▼
        │           ┌────────────────────────────┐
        └───────────┤  Rust Core Engine          │  transactional, safety-critical
                    │  16 services · 6 workers   │  Compliance Governor, audit log,
                    └────────────┬───────────────┘  domain services
                                 │ gRPC / event bus
                                 ▼
                    ┌────────────────────────────┐
                    │  Python Intelligence Engine│  LLM/ML, never touches LinkedIn
                    │  5 services · 3 workers    │  ai-worker (RAG), opportunity-intel,
                    └────────────────────────────┘  kb-intel, voice-intel, scoring-intel
```

- **Rust Core Engine** (`Backend/engine/core/`) — Compliance Governor,
  Integration Gateway, all domain services, audit log. No `unwrap`/`expect`/
  `panic`/`todo`; clippy `-D warnings`.
- **Python Intelligence Engine** (`Backend/engine/intelligence/`) —
  LLM/ML/scoring workloads. Communicates with core **only** via gRPC
  contracts (`proto/lcc/v1/*.proto`) or the event bus (Redis Streams in
  Phase 1–2, Kafka in Phase 3+). Enforced by
  `Backend/infra/ci/scripts/check_boundaries.sh` (CI-gated).
- **Frontend** (`Frontend/`) — never calls LinkedIn directly and never
  bypasses the Governor; every external action routes through the backend
  approval gate (`packages/approval-gate`).

### Non-negotiable invariants

1. A single Compliance Governor issues permit tokens; the gateway verifies
   them (HMAC, ≤60s TTL, single-use `jti`, action-type-bound).
2. The Integration Gateway is the sole holder of LinkedIn credentials.
3. Idempotency by construction — every write-side call carries an idempotency
   key stored in Redis + Postgres.
4. Audit log writes commit in the same DB transaction as the entity write.
5. Row-level security isolates tenants on `app.current_member_id`.

### SLOs

| Component | Target |
|-----------|--------|
| Compliance Governor evaluate p99 | < 200ms |
| API Gateway non-LLM p99 | < 300ms |
| Permit-token verify p99 | < 50ms |
| OAuth token refresh p95 | < 1s |
| Briefing generation p95 | < 3s |
| AI worker draft p95 | < 3s |
| API availability (monthly) | ≥ 99.5% |

---

## Repository layout

```
├── README.md                              ← you are here
├── LCC_AUDIT_REPORT.md                    Full audit: findings, fixes, unfixed items, evidence
├── Backend/                           Rust Core + Python Intelligence monorepo
│   ├── engine/core/                       16 services, 6 workers, 5 admin binaries
│   ├── engine/intelligence/               5 services, 3 workers, 2 admin tools
│   ├── crates/                            12 shared Rust crates
│   ├── packages/                          8 shared Python packages
│   ├── proto/                             gRPC contracts (lcc.v1)
│   ├── schemas/                           SQL migrations, JSON-Schema, OpenAPI
│   ├── config/                            ccfg-*.yaml, features, environments
│   ├── infra/                             Kubernetes, CI, Terraform
│   └── tests/
├── Frontend/                      Next.js + pnpm monorepo
│   ├── apps/web-dashboard/                Next.js 14 App Router + mobile PWA
│   ├── apps/browser-extension/            Manifest V3 Chrome/Firefox extension
│   ├── packages/                          ui, api-types, realtime, approval-gate,
│   │                                      compliance-state, tokens, i18n, test-utils
│   ├── infra/ · tools/ · tests/ · docs/
└── Contract/   Authoritative API contract
    ├── openapi/lcc-api-canonical.yaml     70 paths · 82 operations · 79 schemas
    ├── realtime/lcc-realtime-contract.yaml 5 WS channels · 19 event schemas
    ├── docs/                              endpoint + completeness matrices
    └── canonical-contract-tests/          Rust contract tests
```

---

## Tech stack

| Layer | Choices |
|---|---|
| Core engine | Rust (tokio, tonic/gRPC, sqlx) |
| Intelligence engine | Python (FastAPI, async workers) |
| Frontend | Next.js 14 (App Router), TypeScript strict, Tailwind + shadcn/ui |
| State | TanStack Query v5 (server), Zustand (client), react-hook-form + zod |
| Auth | NextAuth.js (LinkedIn OAuth) |
| Realtime | WebSocket primary · SSE fallback · polling last resort |
| Data | PostgreSQL (RLS multi-tenant) · Redis Streams |
| Testing | Vitest, React Testing Library, Playwright, axe-core, pytest, cargo test |
| Infra | Kubernetes manifests, Terraform, 6 real CI pipelines |

## Getting started

**Backend** (Rust + Python):

```bash
cd Backend
cargo build                      # Core Engine
pytest engine/intelligence/      # Intelligence Engine tests
```

**Frontend** (Node 20.12.2, pnpm ≥ 8.15):

```bash
cd Frontend
pnpm install
pnpm codegen                     # generates @lcc/api-types from the OpenAPI contract
pnpm dev                         # web-dashboard + extension in watch mode
```

The canonical API contract lives in
`Contract/openapi/lcc-api-canonical.yaml` — it is
the single source of truth for frontend↔backend integration.

---

## Project status — read before building on this

This codebase is the direct output of a full end-to-end audit and remediation
pass (`LCC_AUDIT_REPORT.md`). The honest state:

- **0 of 60** documented API endpoints are implemented end-to-end; backend
  routes are scaffolds, and frontend call sites target missing upstreams.
- The Rust workspace had 8+ independent compile blockers at audit time; they
  are addressed in this package but **`cargo build` has not been run** (no Rust
  toolchain in the audit environment). The first real CI run is the next step.
- The 11 original CI workflows were `echo` stubs; 6 real pipelines now exist
  but have not yet run.
- Frontend `tsc --noEmit` passes (0 parse errors); Python AST checks pass on
  all edited files.

**Recommended build order** (full rationale in `LCC_AUDIT_REPORT.md`):

1. Make it build — run CI, clear residual compile blockers.
2. Close the safety loop before any external action: real Governor client in
   the orchestrator, persisted two-reviewer config activation, real audit
   transport, RLS call sites.
3. Run migrations, align `ccfg-*.yaml` to the flat `ComplianceConfig` shape.
4. Reconcile the three API contracts into the one canonical OpenAPI.
5. Implement domain services against it; behavioural tests for all 8 guards.

Every remediation change is commented in-place as `F-AUDIT-nn` at the exact
line it applies to.

## License

See `LICENSE` files in `Backend/` and `Frontend/`.
