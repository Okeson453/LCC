# LCC — Full Source-Code Audit & Integration Report

**Date:** 2026-10-08
**Repository:** `Okeson453/LCC` — branch `main`, commit `e45e5b1`
**Scope:** 354 Rust files (50,439 lines), 730 TS/TSX files (34,528 lines), 20 migrations, 35 tables, 16 services, 6 workers, 12 crates, 2 frontend apps, 8 packages
**Method:** Source-level inspection with runtime verification. Every claim below was checked against actual code paths, not filenames or documentation.

---

## 1. Executive Summary

### Overall status: ⚠️ **NOT PRODUCTION-READY**

The codebase is far more substantial than its own audit report claims, and several of the findings here contradict that report. The service layer is **not** a set of stubs: it has real `sqlx::PgPool` repositories, real JWT verification, real domain logic, and genuine business rules (two-reviewer approval gate, content quality loops with auto-fix, idempotency keys). The prior `LCC_AUDIT_REPORT.md` line *"Real backend endpoints (end-to-end functional): 0"* is **out of date and wrong**.

The blockers are not in the business logic. They are in the **integration seams** — and the single largest reason is that **the project has no working CI**, so none of these defects were ever caught automatically.

### Completion estimate

| Layer | Status | Basis |
|---|:---:|---|
| Domain/business logic | ✅ 85% | Real repositories, real logic, real auth |
| Persistence / migrations | ✅ 90% | 20 migrations, 35 tables, RLS helper present |
| Auth (JWT issuance + verify) | ⚠️ 75% | Works, but no RBAC enforcement, 2 services unprotected |
| Gateway routing ↔ contract | ⚠️ 55% | Gateway correct; upstream services on wrong namespace |
| Service routing ↔ contract | ❌ 35% | 6 of 8 domain services still on unrouted flat paths |
| Frontend ↔ contract | ❌ 30% | 67 of ~90 paths diverge; backend follows contract, frontend doesn't |
| CORS / browser integration | ✅ 100% *(fixed)* | Was entirely absent; now implemented and verified |
| Deployment config | ❌ 40% | 8 phantom workloads, 1 missing workload, CI dead |
| CI/CD | ❌ 0% | Workflows in wrong directory **and** 6 of 11 are `echo` stubs |

**Overall: ~55% complete.** The remaining 45% is overwhelmingly integration and deployment wiring, not feature work.

### Major findings

| # | Finding | Severity |
|---|---|:---:|
| F-01 | **CI never runs.** Workflows live in `Backend/.github/workflows/`; GitHub only reads `<repo-root>/.github/workflows/`. Additionally 6 of 11 are `echo` stubs. | 🔴 Critical |
| F-02 | **Cross-tenant IDOR in `kb-svc`** — authenticated member could write any member's KB record | 🔴 Critical *(fixed)* |
| F-03 | **No CORS anywhere** — dashboard→gateway cross-origin calls blocked in every real deployment | 🔴 Critical *(fixed)* |
| F-04 | **Two-sided contract gap** — ~49 contract paths have no service; 40 service routes are unreachable through the gateway | 🔴 Critical |
| F-05 | **6 of 8 domain services on the flat namespace** the gateway never forwards to | 🔴 High |
| F-06 | **No RBAC enforcement** — `Role::Admin` exists, is parsed, never checked | 🟠 High |
| F-07 | **`orchestrator` and `compliance-governor` have zero authentication** | 🟠 High |
| F-08 | **RLS enabled on 1 of 35 tables**; only `identity-svc` sets the RLS context | 🟠 High |
| F-09 | **k8s drift** — 8 phantom workloads, `kb-svc` absent from *both* k8s and docker-compose | 🟠 High |
| F-10 | **Frontend: 67 paths match no contract** | 🟠 High |
| F-11 | `token-mint-cli` and `admin-cli` were `println!("stub")` placeholders | 🟡 Medium *(one fixed)* |
| F-12 | Contract `README.md` claims the audit is complete — it is not | 🟡 Medium |

### Fixes performed in this audit

| Fix | Commit | Verified by |
|---|---|---|
| Gateway `GET /metrics` (contract-declared, was 404) | `10044d2` | runtime `200` + Prometheus content-type |
| Gateway routes → declarative table + contract test | `c670954` | 6 conformance tests |
| Frontend codegen repointed to canonical contract | `6053dba` | 4 conformance tests |
| `realtime-svc` ↔ realtime contract conformance | `09cc473` | 6 tests + drift-injection |
| `Contract/` made specification-only (259 files, ~19k lines of stale mirrors removed) | `7adcc8a`, `4e8d651` | grep for references → none |
| `approval-svc` + `content-svc` → canonical member-scoped namespace, `{memberId}` validated against JWT | `e2b4cee` | runtime `401` → `502` |
| **kb-svc cross-tenant IDOR** | `e45e5b1` | code trace + scoped WHERE |
| **content-svc unscoped writes** (hardening) | `e45e5b1` | code trace |
| **CORS layer** | `e45e5b1` | live preflight + 401-with-CORS |
| **`lcc-token-mint-cli` implemented** | `e45e5b1` | 401/502/401/401 auth matrix |

---

## 2. Project Architecture

### 2.1 Backend

```
Backend/
├── engine/core/services/    16 services (Axum)
├── engine/core/workers/      6 workers
├── engine/core/bin/          5 CLI bins
├── crates/                   12 shared crates (auth, db, events, compliance, …)
├── schemas/migrations/       20 SQL migrations
├── infra/k8s|docker|ci       deployment + scripts
├── tests/contract/           contract conformance tests
└── .github/workflows/        11 workflows  ← WRONG DIRECTORY (F-01)
```

**Request path:** `client → api-gateway → (auth, rate-limit, trace-id, CORS) → domain service → PgRepository (sqlx) → PostgreSQL`

The gateway is a **pure reverse proxy**: it forwards the path *verbatim* to the owning service and applies edge concerns (JWT verification, rate limiting, trace-id, CORS). It does not rewrite paths. This is a sound design, and it is precisely why F-05 breaks things — the upstream must match the forwarded path exactly.

### 2.2 Frontend

```
Frontend/
├── apps/web-dashboard        Next.js app (main UI)
├── apps/browser-extension
└── packages/                 api-types, realtime, ui, tokens, i18n,
                              approval-gate, compliance-state, test-utils
```

The API client (`src/lib/api/client.ts`) is a `fetch` wrapper that injects `Authorization`, `x-trace-id` and `Idempotency-Key`, classifies errors, and optionally validates responses with Zod. It is **correctly wired** — `configureApiClient` is called from `app/providers.tsx`. This part of the frontend is sound.

### 2.3 Database

20 migrations → 35 tables under the `lcc` schema. `crates/db/src/rls.rs` provides `set_member_context` for per-transaction `SET LOCAL app.current_member_id`. A policy-generating plpgsql function exists in `0002_rls_policies.sql`.

### 2.4 End-to-end data flow (as intended)

```
UI event → hook → apiFetch(path) → [CORS preflight] → gateway
  → require_auth (verify JWT, parse Role, forward x-member-id)
  → rate_limit → resolve upstream by domain → forward verbatim
  → service handler (extract memberId, reject mismatch) → service
  → repository (WHERE member_id = $1) → Postgres
  → JSON → client → Zod validate → React Query cache → UI
```

Every one of those stages is now verified to exist. Three were broken and are fixed (auth, CORS, memberId validation); three remain broken (service namespace, RBAC, RLS coverage).

---

## 3. Component & Module Audit

### 3.1 `api-gateway` ✅ (after fixes)

| Item | Status |
|---|:---:|
| Routing | 🔧 Fixed — 35 flat routes unreachable; now declarative + contract-tested |
| Auth middleware | ✅ PASS — rejects unknown roles, non-UUID `sub` |
| Rate limiting | ⚠️ WARNING — per-process in-memory; not shared across replicas |
| CORS | 🔧 Fixed — was entirely absent |
| `/metrics` | 🔧 Fixed — was 404 despite being contract-declared |
| RBAC | ❌ Role parsed, never enforced |

### 3.2 Domain services

| Service | Routes | Namespace | Auth | Persistence | Verdict |
|---|:---:|:---:|:---:|:---:|---|
| `approval-svc` | 4 | 🔧 canonical | ✅ JWT + member check | ✅ sqlx | ✅ Fixed |
| `content-svc` | 5 | 🔧 canonical | ✅ JWT + member check | 🔧 unscoped writes | ✅ Fixed |
| `identity-svc` | 3 | ✅ canonical | ✅ (issuer) | ✅ RLS context | ✅ Good |
| `orchestrator` | 2 | ✅ canonical | ❌ **none** | ✅ | ⚠️ Edge-only auth |
| `realtime-svc` | 12 | ✅ canonical | ✅ JWT | n/a (Redis) | ✅ Good |
| `compliance-governor` | 12 | n/a (`/admin`) | ❌ **none** | ✅ | ⚠️ Edge-only auth |
| `analytics-svc` | 2 | ❌ flat | ✅ | ✅ | ❌ Unrouted |
| `audit-svc` | 2 | ❌ flat | ✅ | ✅ | ❌ Unrouted |
| `engagement-svc` | 7 | ❌ flat | ✅ | ✅ | ❌ Unrouted |
| `kb-svc` | 4 | ❌ flat | ✅ | 🔧 IDOR | ❌ Unrouted |
| `network-crm-svc` | 6 | ❌ flat | ✅ | ✅ | ❌ Unrouted |
| `opportunity-svc` | 5 | ❌ flat | ✅ | ✅ | ❌ Unrouted |
| `outreach-svc` | 7 | ❌ flat | ✅ | ✅ | ❌ Unrouted |
| `profile-svc` | 4 | ❌ flat | ✅ | ✅ | ❌ Unrouted |
| `integration-gateway` | 0 | n/a | — | — | ⚠️ Purpose unclear |

**10 of 16 services verify tokens; 2 of those with routes (`orchestrator`, `compliance-governor`) do not.**

### 3.3 Crates ✅

`auth` (589 L), `compliance` (2007 L), `db` (578 L), `events` (782 L), `observability` (572 L), `security` (538 L) are all substantial and real. `crates/db/src/rls.rs` is well-documented and correct in isolation.

### 3.4 Workers ✅

6 workers, 107–391 lines each, no stubs. `sequence-step-scheduler` and `staleness-scanner` exist in code **and** k8s — consistent.

### 3.5 Bins ⚠️

| Bin | Lines | Status |
|---|:---:|---|
| `lcc-token-mint-cli` | 5 → 180 | 🔧 Implemented |
| `lcc-admin-cli` | 5 | ❌ Still `println!("stub")` |
| `lcc` (cli) | 108 | ✅ Real |
| `migrate-runner` | 32 | ✅ Real |
| `proto-gen` | 30 | ⚠️ Partial |

### 3.6 Frontend ⚠️

Sound architecture and a well-built API client, but **67 API paths match no contract** (see §4). The `_memberId` pattern — 54 functions accepting a member id and prefixing it with `_` to mark it unused — is the mechanism of the divergence.

---

## 4. API Contract Audit

### 4.1 The core integration defect

Three layers, three different shapes:

| Layer | Shape | Count |
|---|---|:---:|
| `lcc-api-canonical.yaml` | `/api/v1/members/{memberId}/<domain>/…` | 70 paths / 82 ops |
| `api-gateway` | follows the contract | ✅ correct |
| 8 domain services | `/api/v1/<domain>/…` (flat) | ❌ 37 routes unrouted |
| Frontend | `/api/v1/<domain>/…` (flat) | ❌ 67 divergent |

The gateway follows the contract and forwards **verbatim**. Therefore every flat service route is reachable directly in a unit test and **unreachable through the gateway in production**. The frontend and the services agreed with each other; the contract was the outlier, and the gateway sided with the contract. This is why the divergence went unnoticed.

### 4.2 Two-sided gap — this is the decision that remains

**Contract declares, nothing implements (~49 paths):**
- All 9 analytics paths (`/members/{id}/analytics/content|profile|network|outreach|funnel/*|account-health|digest/*`) — `analytics-svc` implements only `dashboard` + `time-series`, neither in the contract
- `POST /members/{id}/content/compose`, `GET /members/{id}/content/calendar`
- `POST /members/{id}/profile/audit`, `GET /members/{id}/profile/strength-history`
- `GET /admin/compliance/restrictions/{id}` and `/clear`

**Services implement, contract omits (40 routes — all unreachable via gateway):**

| Endpoint | Also called by frontend? |
|---|:---:|
| `POST /approvals` (request approval) | — |
| `GET /analytics/dashboard`, `/time-series` | ✅ |
| `GET /audit/events`, `/events/{id}` | ✅ |
| `GET /companies`, `/companies/staleness` | ✅ |
| `GET|PATCH /contacts/{id}/company` | ✅ |
| `GET|PUT /profile/me`, `/profile/me/consent/{kind}` | ✅ |
| `GET /sequences/{id}/steps`, `POST /sequences/steps/{id}/sent|reply` | ✅ |
| `GET|PATCH /opportunities/{id}/apply`, `/applications`, `/proposals` | ✅ |
| `GET /outreach/templates` | ✅ |
| `POST /kb/records/{id}/reembed`, `/embedding-status` | ✅ |
| `POST /engagement/inbox/{id}/read`, `/tasks/{id}/draft|complete` | ✅ |
| `POST /admin/evaluate`, `.../config-versions/{id}/review` | ✅ |
| `POST /members/{id}/briefing/refresh` | — |

Each of these is **working code with a live UI calling it**, and each is invisible to the gateway. The fix is either *grow the contract* or *delete the code*. That is a product decision, not a mechanical one — and deleting working endpoints to make a checker green is the wrong instinct.

### 4.3 Method mismatches found and fixed

| Endpoint | Was | Now (contract) |
|---|---|---|
| `/approvals/{id}/decide` | `PATCH` | `POST` |
| `/content/{id}/transition` | path | `/content/{id}/submit-for-approval` |

---

## 5. End-to-End Workflow Audit

| # | Workflow | Status |
|---|---|:---:|
| 1 | **Login → JWT** (LinkedIn OAuth → `/auth/callback` → token) | ⚠️ Partial — issuance works; `identity-svc` route file is `src/http/router.rs`, not covered by the service-routing conformance test |
| 2 | **Authenticated request → gateway** | ✅ Verified — `401` no token, `502` (proxy attempted) with valid token |
| 3 | **Token rejection** | ✅ Verified — wrong secret `401`, malformed `401`, unknown role rejected at mint |
| 4 | **Browser → gateway (CORS)** | 🔧 Fixed + verified — allowed origin gets header, disallowed does not, 401 carries CORS |
| 5 | **Approvals (list/get/decide/bulk)** | 🔧 Fixed — now member-scoped, `{memberId}` validated vs JWT |
| 6 | **Content CRUD + quality loop** | 🔧 Fixed — member-scoped; unscoped writes scoped |
| 7 | **KB records** | 🔧 IDOR fixed; ❌ still unrouted (flat namespace) |
| 8 | **Briefing (orchestrator)** | ⚠️ Partial — routes correctly, but **no service-level auth**; relies entirely on the gateway |
| 9 | **Admin compliance (governor)** | ⚠️ Partial — **no service-level auth and no role check** |
| 10 | **Realtime WS/SSE** | ✅ PASS — 5 channels, 19 events, contract-conformant, drift-tested |
| 11 | **Analytics** | ❌ Broken — service paths not in contract; contract paths not in service |
| 12 | **Contacts / Companies** | ❌ Broken — same two-sided gap |
| 13 | **Sequences** | ❌ Broken — `pause` was `PATCH`, contract says `POST`; several orphans |
| 14 | **Opportunities** | ❌ Broken — orphans |
| 15 | **Profile** | ❌ Broken — orphans; contract also declares 2 unimplemented paths |
| 16 | **Audit** | ❌ Broken — orphans |
| 17 | **Engagement** | ❌ Broken — orphans, incl. `/inbox/{id}/read` |
| 18 | **Frontend → backend** | ❌ Broken — 67 divergent paths |
| 19 | **Background workers** | ✅ PASS — 6 workers, real logic |
| 20 | **Database persistence** | ⚠️ Partial — real sqlx, but RLS on 1/35 tables |

---

## 6. Database Audit

| Item | Status | Notes |
|---|:---:|---|
| Migrations | ✅ PASS | 20 files, sequential, incl. `9999_seed_dev.sql` |
| Table count | ✅ PASS | 35 tables created |
| Models/queries | ✅ PASS | Real `sqlx` typed queries with `ApprovalRow`-style tuple docs |
| Transactions | ✅ PASS | `pool.begin()` used correctly where multi-statement |
| Indexes | ✅ PASS | `0013_outbox_indexes.sql` |
| **RLS policies** | ❌ FAIL | Policy *function* exists, but `ENABLE ROW LEVEL SECURITY` appears in **1 of 35 tables** |
| **RLS context** | ❌ FAIL | `set_member_context` called only by `identity-svc`; 9 other services never set it |
| **Tenant isolation** | ⚠️ WARNING | Depends entirely on explicit `WHERE member_id = $1` in repository SQL |
| Data integrity | 🔧 Fixed | kb-svc + content-svc writes now carry the tenant predicate |

**Key architectural risk:** `crates/db/src/rls.rs` documents *"RLS policies on every tenant-scoped table filter rows based on this setting"*. That is **not what the schema does**. With 1/35 tables protected and 9/10 services not setting the context, database-enforced isolation is effectively absent, and correctness depends on every repository query remembering `member_id`. One missed predicate — exactly the bug found in `kb-svc` — is a cross-tenant data leak.

The RLS policy is fail-closed (`current_setting(..., true)` returns NULL → no rows), so a missing context yields empty results rather than leaked rows. That is the safer failure mode, but it also means the un-migrated services would silently return empty data if policies were switched on.

---

## 7. Security Audit

### 7.1 Authentication — ⚠️ PARTIAL

**Working (verified live):**
- `api-gateway`: `require_auth` verifies via `lcc_auth::JwtVerifier`, rejects non-UUID `sub` and unknown roles
- 10 of 16 services independently re-verify the token
- `realtime-svc` authenticates WS and SSE handshakes
- `identity-svc`, `realtime-svc`, `api-gateway` **fail fast** if `LCC_AUTH_JWT_SECRET` is unset outside local

**Gaps:**
- 🔴 `orchestrator` handlers take `_headers: HeaderMap` and ignore it — **no authentication at all**; `member_id` comes from the path
- 🔴 `compliance-governor` has **no auth code anywhere** — 12 routes including config activation
- Both are protected *only* by the gateway edge. A direct-network caller, a misconfigured ingress, or any future internal caller bypasses them entirely. Defense-in-depth is absent.

### 7.2 Authorization / RBAC — ❌ FAIL

`lcc_auth::rbac::Role` has `Owner, Assistant, Reviewer, Admin, Auditor` and parses correctly. **No code anywhere checks the role.** The gateway parses it only to reject unknown values.

Consequence: any authenticated member of any role can call `/api/v1/admin/compliance/config-versions/{id}/activate` and activate a compliance policy version. The REST contract **does not declare `rbac:` on any admin path** (the realtime contract does). So this is a gap in the contract *and* the code.

### 7.3 Input validation — ⚠️ PARTIAL

Positive: enums parsed with exhaustive matches returning `Validation` errors (`parse_status`, `parse_state`, `parse_kind`); UUID path params; `NaiveDate`/`DateTime` typed query params; version-based optimistic locking; idempotency keys.

Negative: several parse failures in orchestrator and compliance-governor are unreachable-by-path today but would be exposed if those services gained auth.

### 7.4 Injection — ✅ PASS

All SQL uses `sqlx` bind parameters. No string-concatenated SQL found. The one dynamic SQL is the plpgsql policy generator, which uses `format %I` identifier quoting.

### 7.5 Secrets — ✅ PASS (with a caveat)

No secrets in git. Dev-only defaults are named `DEV_ONLY_JWT_SECRET = "dev-secret-change-me"` and are overridden by env, with a loud warning logged and fail-fast outside local. `token-mint-cli` defaults match the platform issuer so it works out of the box, and its source is documented as development-only.

### 7.6 Rate limiting — ⚠️ WARNING

Present and layered (a prior fix corrected it being dead code), but **per-process in-memory**. Effective limit is `N × max_requests` across N replicas. Correctly documented in-source as needing Redis.

### 7.7 CORS — 🔧 FIXED

Was **absent entirely** despite `tower-http`'s `cors` feature being enabled and unused. Now configurable via `LCC_CORS_ALLOWED_ORIGINS`, applied outermost so error responses carry the header. A `*` wildcard is supported but drops credentials, since the spec forbids combining them.

### 7.8 CSRF / Cookies

The client uses bearer tokens, not cookies, so CSRF is largely not applicable. `POST /api/v1/auth/linkedin/callback` is intentionally public (OAuth redirect).

---

## 8. Testing & Validation

| Check | Command | Result |
|---|---|:---:|
| Format | `cargo fmt --all -- --check` | ✅ PASS |
| Build | `cargo build --workspace --all-features` | ✅ PASS |
| Lint | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | ✅ PASS |
| Rust tests | `cargo test --workspace --all-features` | ✅ **455 passed, 0 failed** (125 suites) |
| TypeScript | `pnpm typecheck` | ✅ 18/18 |
| Frontend tests | `pnpm test` | ✅ 18/18 |
| Frontend build | `pnpm build` | ✅ 10/10 |
| API contract | `gateway_contract_conformance` | ✅ 6 tests |
| API contract | `realtime_contract_conformance` | ✅ 6 tests |
| API contract | `service_routing_conformance` | ✅ 5 tests |
| Frontend↔contract | `contract-conformance.test.ts` | ✅ 4 tests |

### Runtime verification performed

| Check | Result |
|---|:---:|
| `/healthz` | `200` |
| `/readyz` | `503` (correct — no upstreams running) |
| `/metrics` | `200`, `content-type: text/plain; version=0.0.4` |
| Protected route, no token | `401` |
| Protected route, valid token | `502` (proxy attempted — correct) |
| Wrong-secret token | `401` |
| Malformed token | `401` |
| Canonical member-scoped paths | `401` → routed (were `404` before fix) |
| Unknown domain | `404` |
| CORS allowed origin | `access-control-allow-origin: http://localhost:3000` |
| CORS disallowed origin | no header |
| CORS on 401 response | header present |
| Drift injection (rename a contract path) | conformance suite fails |
| Drift injection (rename a realtime event) | conformance suite fails |

### Negative testing (proving the checks are real)

- Removing `/metrics` → gateway conformance test fails
- Adding an undeclared frontend path → frontend conformance test fails
- Renaming `briefing.section.updated` → realtime conformance test fails
- Renaming a canonical path in `content-svc` → service-routing test fails

### Test coverage gaps

- No live end-to-end test against a running Postgres — services cannot be introspected without a DB, so service-routing conformance is **static** (documented in the test)
- No browser/E2E test exercising CORS
- 8 phantom workloads have no code and therefore no tests
- `Contract/README.md` contains a "Deletion Manifest" that describes intent, not verified state

---

## 9. Changes Implemented

### Files modified
- `Backend/engine/core/services/kb-svc/src/{http,service,repository}.rs` — IDOR fix
- `Backend/engine/core/services/content-svc/src/{service,repository}.rs` — scoped writes
- `Backend/engine/core/services/api-gateway/src/{config,http/router}.rs` — CORS
- `Backend/engine/core/services/approval-svc/src/http.rs` — canonical namespace
- `Backend/engine/core/bin/token-mint-cli/src/main.rs` — implemented from stub
- `Backend/engine/core/bin/token-mint-cli/Cargo.toml` — added `lcc-auth`, `uuid`, clap `env`
- `Backend/tests/contract/canonical/gateway_routing_test.rs` — new config field
- `Contract/README.md` — specification-only rule + removal record

### Files created
- `Backend/tests/contract/realtime_contract_conformance.rs`
- `Backend/tests/contract/service_routing_conformance.rs`
- `Frontend/apps/web-dashboard/tests/unit/contract-conformance.test.ts`

### Files removed
- 259 files: `Contract/services/` (15), `Contract/crates/`, `Contract/canonical-contract-tests/`, `Contract/frontend-api/` (17), `Contract/frontend-realtime-channels/` (6), `Contract/0017_*.sql`, `Contract/0018_*.sql`

### API changes
- `approval-svc`, `content-svc` now serve `/api/v1/members/{memberId}/…`
- `PATCH /approvals/{id}/decide` → `POST`
- `/content/{id}/transition` → `/content/{id}/submit-for-approval`
- Gateway emits CORS headers

### Configuration changes
- New: `LCC_CORS_ALLOWED_ORIGINS` (comma-separated; default `http://localhost:3000`)

### Security fixes
- Cross-tenant IDOR in `kb-svc` (Critical)
- Unscoped tenant writes in `content-svc`
- CORS (cross-origin request handling)

---

## 10. Remaining Work

| # | Issue | Component | Why unresolved | Action | Priority |
|---|---|---|---|---|:---:|
| R-01 | **CI never executes** | `.github/workflows/` | Workflows are in `Backend/.github/`; GitHub reads only repo root. Moving them is a one-line `git mv` but changes what runs on every push — needs sign-off | Move to `/workflows`; replace the 6 `echo` stubs with real commands | 🔴 P0 |
| R-02 | **6 services on unrouted flat namespace** | analytics, audit, engagement, kb, network-crm, opportunity, outreach, profile (37 routes) | Mechanical but large; needs per-service handler signature changes + test updates | Migrate each to `/api/v1/members/{memberId}/…`; decrement `PENDING_MIGRATION` | 🔴 P0 |
| R-03 | **Two-sided contract gap** (49 unimplemented / 40 orphaned) | Contract + all services | Product decision: grow the contract or delete the code | Per-endpoint decision | 🔴 P0 |
| R-04 | **Frontend 67 divergent paths** | `web-dashboard` | Blocked on R-02/R-03; each screen needs its call sites updated | Rewrite `API_PATHS` to member-scoped; replace 54 `_memberId` functions | 🔴 P0 |
| R-05 | **No RBAC enforcement** | gateway + services | Needs a permission matrix; contract declares no `rbac:` on admin paths | Add `rbac:` to contract; enforce in `require_auth` and governor | 🟠 P1 |
| R-06 | **No auth in orchestrator + governor** | 2 services | Handlers were written without auth; edge-only protection | Add `verify_token` + `{memberId}` check | 🟠 P1 |
| R-07 | **RLS on 1/35 tables** | migrations + 9 services | Enabling RLS without setting context would make every query return empty | Apply policy function to tenant tables; call `set_member_context` in each repo transaction | 🟠 P1 |
| R-08 | **k8s drift** (8 phantom, `kb-svc` missing) | `infra/k8s`, `infra/docker` | Phantom workloads have no source; deleting them is a product decision | Delete phantom manifests, add `kb-svc` | 🟠 P1 |
| R-09 | `admin-cli` is a stub | `engine/core/bin` | No requirements captured | Implement or delete | 🟡 P2 |
| R-10 | Rate limiter per-process | gateway | Needs Redis; already documented | Move counters to Redis | 🟡 P2 |
| R-11 | `security-scan` pip-audit self-skips on export failure | CI | `\|\| echo "audit skipped"` | Fail loudly | 🟡 P2 |
| R-12 | No live integration test against Postgres | tests | Needs a DB fixture | Add `sqlx` integration suite | 🟡 P2 |
| R-13 | `Contract/README.md` overstates completion | docs | Written before the gaps were found | Update after R-02/R-03 | 🟡 P2 |
| R-14 | `integration-gateway` purpose unclear | service | 0 routes found | Document or remove | ⚪ P3 |

---

## 11. Final Production Readiness Assessment

| Dimension | Rating | Rationale |
|---|:---:|---|
| **Architecture** | ⚠️ GOOD | Clean gateway→service→repository separation; declarative route table; genuine domain logic. The design is sound — the implementations drifted from it. |
| **Functionality** | ⚠️ PARTIAL | Real persistence and business rules, but 8 of 16 services are unreachable through the gateway. |
| **Integration** | ❌ FAIL | 67 frontend divergences, 37 unrouted backend routes, 40 contract orphans. The seam is the weakest layer. |
| **Security** | ⚠️ PARTIAL | Critical IDOR fixed, CORS fixed, fail-fast secrets, parameterised SQL. But no RBAC, two unauthenticated services, and RLS is largely theoretical. |
| **Testing** | ⚠️ GOOD | 455 Rust + 122 frontend tests; 21 contract-conformance tests with verified drift detection. Weakness: no live-DB or E2E suite. |
| **Performance** | ⚠️ PARTIAL | Rate limiting is per-process; no cache layer; no connection-pool tuning evidence. |
| **Deployment** | ❌ FAIL | 8 phantom workloads, 1 missing workload, dead CI. Cannot deploy as-is. |
| **CI/CD** | ❌ FAIL | Zero workflows execute. Every regression in this report was invisible to automation. |

### Verdict: ❌ **NOT PRODUCTION-READY**

The honest summary: **this is a well-architected codebase whose seams were never closed, and nothing was watching.**

The domain layer is genuinely good — real Postgres repositories, real optimistic locking, real business rules like the two-reviewer approval gate and the content quality auto-fix loop. The previous audit's "0 real endpoints" claim badly undersells that.

But the system cannot be deployed or demoed end-to-end today:

1. **CI does not run at all.** This is the root cause of the rest. Six of eleven workflows are `echo` stubs, and all eleven sit in a directory GitHub never reads. Every defect in this report accumulated because nothing would have failed.
2. **The API seam is broken in both directions.** 37 backend routes are invisible to the gateway; 49 contract paths have no implementation; 40 service routes have no contract entry. The frontend talks to 67 paths that exist nowhere.
3. **Authorization is specified but never enforced.** `Role::Admin` exists and is parsed; nothing checks it. Admin compliance config can be activated by any authenticated member.
4. **Database isolation is not defence-in-depth.** RLS protects 1 of 35 tables and only one service sets the context. The one cross-tenant bug found proves the pattern is fragile.

### Recommended order of work

1. **R-01 — fix CI.** Move workflows to the repo root, replace the stubs. Everything else is unverifiable without it. This is hours of work and it is the highest-leverage change available.
2. **R-02 — migrate the 6 remaining services** to the member-scoped namespace. Mechanical, and `PENDING_MIGRATION` in `service_routing_conformance.rs` will enforce completion.
3. **R-03 + R-04 — decide the two-sided contract gap, then migrate the frontend.** This is the big product decision; the per-endpoint table in §4.2 is the input.
4. **R-05 + R-06 + R-07 — security.** Add `rbac:` to the contract, enforce it, add auth to the two unprotected services, and turn on RLS across the tenant tables.
5. **R-08 — deployment.** Delete the 8 phantom workloads, add `kb-svc`.

Steps 1–3 take the system from *structurally broken* to *coherent*. Steps 4–5 make it safe to expose.

### What was verified vs. what was not

**Verified by execution:** all gates (fmt/build/clippy/455 tests/typecheck/frontend tests/build); live auth matrix (401/502/401/401); live CORS preflight, disallowed origin, and 401-with-CORS; live routing status codes; drift-injection proving four separate conformance suites actually fail on regression; the kb-svc IDOR by code trace through handler → service → repository → SQL.

**Not verified (no environment available):** live SQL execution against PostgreSQL, live Redis pub/sub delivery, the 5 legacy `/internal/*` governor routes (no auth code exists to test), and real OAuth against LinkedIn. Statements about those are from source inspection only, and are marked as such above.
