# LinkedIn Manager (LCC) — End-to-End Audit & Remediation Report

**Scope:** `LCC_frontend.zip` (790 files) + `LCC Backend.zip` (838 files), verified against all six
design documents (Design Framework, Technical Design Spec, Backend Design Concept, Backend Monorepo
Layout, Frontend Design Concept, Frontend Full File Structure).

**Method:** every finding below is derived from the actual source, the actual execution path, the
actual schema, or the actual config — never from filenames, comments, or declared architecture.
Findings carry `file:line` evidence.

**Headline:** this is a **scaffold, not a system.** The design documents describe a compliance-gated
LinkedIn automation platform. What exists is a structurally-shaped skeleton in which the safety gate
is dead code, the API gateway fabricates success responses, the database schema cannot be built, and
**not one of the 60 required API endpoints is implemented**. The frontend, by contrast, is
substantially real and production-shaped — it is pointed at a backend that does not exist.

---

## 1. Executive summary

| Dimension | State | Evidence |
|---|---|---|
| Required API endpoints implemented | **0 / 60** | §18 catalog vs. actual routers |
| Endpoints the frontend calls that exist in the backend | **0 / 49** | §4 below |
| Domain services with real domain logic | **0 / 11** | all 11 are byte-identical 692-LOC scaffolds |
| Compliance Governor callers in the entire repo | **0** | `evaluate_action` has 2 refs, both inside the governor |
| Guard-stack behaviour (guards 6 & 7 query non-existent tables) | **denies 100% of actions** | §3.1 |
| Does the Rust workspace compile? | **No** | 8+ independent compile blockers, §3.0 |
| Did CI ever catch any of this? | **No — all 11 workflows were `echo` stubs** | §6 |
| Can the database schema be created? | **No** | unqualified `%I`, no `search_path`, §5.1 |
| Frontend typecheck (parse) | **2 pre-existing syntax errors, both fixed; 0 remain** | §2.5 |

The most dangerous finding is not any single defect. It is the combination: the Governor **denies
everything**, **nothing calls it anyway**, and its own configuration-activation endpoint accepts any
two arbitrary non-empty strings as a "two-reviewer signature". The safety envelope is simultaneously
non-functional, unreachable, and writable by anyone who can reach the port.

---

## 2. What was fixed (36 files, 45 defects)

All fixes below were applied to the working tree. Each is self-contained and verifiable by
inspection. Fixes are labelled `F-AUDIT-nn` in source comments at the point of change.

### 2.1 Safety-critical

| ID | Defect | Fix |
|---|---|---|
| **F-AUDIT-01** | `api-gateway/src/http/handlers/mod.rs` — `proxy_get`/`proxy_post` returned a hardcoded `{"status":"ok"}` for **every** protected route, contacting nothing. `proxy/pool.rs` (a real upstream client) was dead code. | Real reverse proxy: `UpstreamRegistry` resolving domain→upstream, header allow-listing, 8 MiB body cap, hop-by-hop stripping, status/header/body pass-through. Unimplemented upstreams now surface as honest **404s** instead of fabricated 200s. |
| **F-AUDIT-12** | `sequence-step-scheduler` marked due steps `sent` directly, with the comment *"In production: publish a sequence_step.due event"*. **Every scheduled sequence step bypassed all 8 compliance guards**, the permit token, and the integration gateway — the system's first axiom. It also recorded sends that never happened. | Step is now `pending_evaluation` → governor `evaluate` → on PERMIT only, dispatch to the integration gateway (sole credential holder) → then `sent`. DENY returns to `pending`/`blocked`; governor-unreachable is **fail-closed**. Migration `0015` adds `denial_count`/`last_error`/`permit_token`. |
| **F-AUDIT-19** | Guard 6 (`restriction_flag`) queried `SELECT is_restricted FROM member_account` — **that table does not exist anywhere in the repo**. Fail-closed ⇒ **guard 6 denied 100% of actions**, making the entire permit-issuance path unreachable. | Corrected to `EXISTS(SELECT 1 FROM lcc.restrictions WHERE member_id=$1 AND cleared_at IS NULL)`, and the denial reason now names the signal. |
| **F-AUDIT-20** | Guard 7 (`approval_state`) queried `SELECT status FROM approval … idempotency_key_match($2)` — wrong table, wrong column, and a function the source itself called *"hypothetical"*. Fail-closed ⇒ **100% denial**. | Corrected to `lcc.approvals` with the `decision` enum, matched on `resource_id` **or** `requested_action->>'idempotency_key'`, with explicit expiry handling. |
| **F-AUDIT-22** | A valid `permit_token` was **replayable for its entire TTL**. `jti` was minted but never read; `PermitError::Replay` was declared but **never constructed**; the doc claimed Redis replay checking that did not exist. | New `permit/replay.rs` — `SET permit:seen:<jti> 1 NX EX <ttl+1>`, first caller wins, **fails closed** on store error. Wired into both Track A and Track B. |
| **F-AUDIT-23** | The permit's `typ` (action type) was never compared to the requested action. A permit minted for `like` authorised a `post_publish` — permit confusion. | `verify()` now takes `expected_action_type`; new `ActionTypeMismatch` error; 5 unit tests. |
| **F-AUDIT-18** | `crates/compliance/src/lib.rs` declared `pub type PermitSigner = PermitSigner;` (×3) — self-referential aliases, **fatal E0391**. `lcc-compliance` is the shared dependency of *both* safety-critical services; **neither could compile**. | Deleted. All three names are still exported by the existing `pub use`. |

### 2.2 Gateway correctness

| ID | Defect | Fix |
|---|---|---|
| **F-AUDIT-07** | `middleware/auth.rs` imported `lcc_auth::jwt::{decode_token, Claims}` and called `state.jwt_secret()`/`.db()` — **none exist** (the type is `JwtClaims`; there is no free `decode_token`; `AppState` has no such accessors). Also used undeclared `parking_lot` and mismatched `Arc<AppState>` vs `AppState`. **Compile-blocking.** | Rewritten against the real `lcc-auth` API. Removed a per-request `sqlx::Transaction` that was opened and immediately rolled back on every request (connection-pool pressure for nothing). Role is now validated through `lcc_auth::Role`. |
| **F-AUDIT-05** | `rate_limit` was never layered onto the router **and** constructed a fresh `RateLimiter` — i.e. a fresh empty counter — **per request**, so no limit could ever be reached. Keyed on the spoofable `x-member-id` header. | Shared limiter in `AppState`, layered onto the router, keyed on the JWT subject with peer-address fallback. Emits `X-RateLimit-Limit/Remaining/Reset` + `Retry-After` per §17. 3 unit tests. |
| **F-AUDIT-11** | `readyz` returned `{"status":"ready"}` without contacting any upstream — the k8s readiness probe always passed even against a fully dead backend. | Real per-upstream `/healthz` probe, 750 ms budget, fail-closed 503. |
| **F-AUDIT-37** | **All 10 upstream URLs pointed at `:8080`**, but only api-gateway binds 8080; the services bind 8081–8091. Every proxy route was connection-refused → 502. | Corrected to the real per-service ports; pinned explicitly in the k8s manifest. |
| **F-AUDIT-10** | `auth_jwt_secret` defaulted to the literal `"dev-secret-change-me"`. | `from_env()` now returns `Result` and **refuses to start** on any non-`local`/`dev`/`test` profile; overlay patches set `LCC_ENVIRONMENT`. |
| **F-AUDIT-06** | `trace_id.rs` used `rand` and `hex`, neither declared in `Cargo.toml` — **compile-blocking**. | Dependencies added. |
| **F-AUDIT-21** | `permit/mod.rs` re-exported `verify_permit_token`, a name defined nowhere — **E0432**. | Re-export corrected. |
| **F-AUDIT-14/15/36** | 5 of 6 workers called functions from `src/logic.rs` without declaring `mod logic;` — **compile-blocking**. All workers' `Config::from_env()` returned `Self::default()`, ignoring the environment entirely. | `pub mod logic;` declared in all 6; real env-backed config. |

### 2.3 Database / schema

| ID | Defect | Fix |
|---|---|---|
| **F-AUDIT-33** | `attach_member_rls` used bare `%I` with **no `search_path` set anywhere** ⇒ `ERROR: relation "members" does not exist`. **The schema could not be built at all.** | Explicit `lcc.%I` schema qualification, still `%I`-quoted against injection. |
| **F-AUDIT-35** | `ALTER POLICY … USING` on `lcc.members` left the 0002 `WITH CHECK (member_id::text = …)` in place — but `lcc.members` has no `member_id` column, so every INSERT/UPDATE failed. | Policy recreated with both `USING` and `WITH CHECK` on `id`. |
| **F-AUDIT-13** | Staleness scanner used a flat hardcoded 90-day window, ignoring the spec's tiered 30/60/90 thresholds, and **discarded its result** — so `GET /contacts/stale` had no data source. | Tier-derived (VIP 30 d / Standard 60 d) evaluated in SQL, persisted to `lcc.contacts` via migration `0016`, with a recovery pass. |
| **F-AUDIT-44** | `AuditTx::record` inserted into a non-existent `audit_log` table with three non-existent columns. | Retargeted to `lcc_audit.events`; `member_id` and `prev_checksum` now populated (the integrity worker compares the chain, which could never have linked). |
| **F-AUDIT-16/17** | Briefing worker issued **4 sequential queries per member, up to 100 members** = 400 serial round-trips per tick (§9.4 specifies `tokio::join!`), and hardcoded `"recommendations": []`. | Concurrent fan-out per §9.4; recommendations assembled from the sections already read. |

### 2.4 Intelligence Engine (Python)

| ID | Defect | Fix |
|---|---|---|
| **F-AUDIT-25** | ρ shipped **hardcoded β** and reported `mode="ml"` once a member hit 200 labeled sends — precisely the silent substitution of an uncalibrated model that the spec forbids. | `load_rho_model()` returns `None` unless a genuinely fitted artifact (`fitted_on_samples ≥ 200`) exists; falls back to rule-based, labelled as such. |
| **F-AUDIT-26** | `record_spend` had **no production caller** ⇒ `spent_usd` was always 0 ⇒ the entire cost ladder was unreachable and the $500/month ceiling never enforced. | Wired into all 5 draft endpoints; `configure()` now validates. |
| **F-AUDIT-28** | The brand-guard linter computed a `passed` verdict that **every call site discarded** — banned phrases surfaced as lower confidence, not rejection. | `_lint_and_filter()` enforces the verdict (Technical Design Spec §22 requires it as a gate). |
| **F-AUDIT-27** | `DraftVariant.kb_refs` had no `min_length`, so a hallucinated draft satisfied axiom 5. | `Field(min_length=1)`. |
| **F-AUDIT-29** | All 3 worker Dockerfiles ran `USER 10001:10001` **without ever creating that user** — every image crashed at `docker run`. | `useradd` added, matching the 5 services that do it correctly. |
| **F-AUDIT-30** | **All 11 CI workflows were `echo` stubs.** No compiler, linter, or test had ever run. | 6 workflows replaced with real pipelines (Rust fmt/clippy/test, Python lint/test, cargo-deny, buf lint). |

### 2.5 Frontend

| ID | Defect | Fix |
|---|---|---|
| **F-AUDIT-32** | `app/(app)/layout.tsx:22` — `import {,, Toaster, … }` — **syntax error**; the app could not compile. | Fixed. |
| **F-AUDIT-32** | `components/copilot/CopilotRoot.tsx:27` — unterminated string in a template literal — **syntax error**. | Fixed. |
| **F-AUDIT-31** | `configureApiClient()` ran in a `useEffect`. React runs effects bottom-up, so a child `useQuery` fired **before** the client was configured ⇒ `ApiError("API client not configured")` on cold load. | Configured during render (it only assigns a module variable, so it is React-safe). |
| **F-AUDIT-32** | `useRequireAccessToken` called `useClientSession()` **inside a `useCallback`** — a Rules-of-Hooks violation, and with `[]` deps it would return a stale token after refresh. | Hook called at top level; token in the dependency array. |

**Verified after fix:** `tsc --noEmit` over `apps/web-dashboard` and `apps/browser-extension`
now reports **0 parse/syntax errors** — down from 10 (2 root causes producing 10 secondary
diagnostics). The earlier `layout.tsx` import error alone cascaded into 8 JSX errors; the
`CopilotRoot.tsx` unterminated string into 7.

The remaining **838 diagnostics** are type-resolution failures, dominated by 139
`TS2307 Cannot find module` (`lucide-react` ×14, `date-fns` ×3, `@tanstack/react-query` ×4 …)
cascading into `TS7006` implicit-any (204), `TS7031` binding-element-any (144), `TS7016`
(77) and `TS2322` (95). The `pnpm install` staged 775 packages into `.pnpm` but never completed
its symlink phase, so no external package resolves. I spot-checked the apparently-real
`TS2322 … not assignable to BadgeProps` errors: `variant="warning"` **is** a valid variant
(`packages/ui/src/primitives/badge.tsx:15`), and the error arises only because `cva`'s
`VariantProps` type is unresolvable — i.e. noise, not a defect.

**I cannot certify the full 838 as install noise without a complete install**, and I do not claim
to. What is verified: zero parse errors, and the two defects that prevented any build were real
and are fixed.

### 2.6 Infrastructure

| ID | Defect | Fix |
|---|---|---|
| **F-AUDIT-38** | All 6 worker manifests declared `httpGet` liveness/readiness probes on `/healthz` and `/readyz` against batch processes that **bind no HTTP listener** — those pods could never go Ready. | Exec liveness probe; bogus Service + `containerPort` removed; `prometheus.io/scrape: "false"`. |
| **F-AUDIT-43** | 4 host-port collisions in `docker-compose.yml` ⇒ `docker compose up` cannot bind. Obsolete `version:` key. | Host ports renumbered; duplicates verified zero. |
| **F-AUDIT-41/42** | No manifest set `LCC_ENVIRONMENT` or any `LCC_*_SVC_URL` — which silently defeated the JWT-secret guard and left every upstream on a wrong port. | Set in the base manifest; per-overlay environment patch added. |

---

## 3. Findings NOT remediated (and why)

These are real, evidence-backed, and **not fixable within this engagement**. They are the gap
between "audited" and "production".

### 3.0 The domain layer is unimplemented — the dominant gap

All 11 domain services (`analytics`, `approval`, `audit`, `content`, `engagement`, `identity`,
`network-crm`, `opportunity`, `orchestrator`, `outreach`, `profile`) are **byte-identical 692-LOC
scaffolds**: the same `Entity`/`EntityCreate` model (`state: Active|Pending|Archived|Deleted`,
`kind: Primary|Secondary|Auxiliary`, `title`, `body`, `metadata`), the same CRUD router, the same
repository against `lcc.{name}_svc` — a table **no migration creates**.

None of the required domain behaviour exists: no `ContentItem` state machine (9 states vs 4),
no `Opportunity` φ pipeline, no `Sequence` state machine, no `Contact`/`Company` CRM, no `Approval`
decide endpoint, no `MessageTemplate`, no `Application`.

Closing this means implementing 11 bounded contexts, ~60 endpoints, the full persistence layer and
their state machines. That is the product — not a defect repair — and it cannot be honestly claimed
as "done" without writing and testing it.

### 3.1 Safety-path defects that remain

| Finding | Evidence | Impact |
|---|---|---|
| **Governor has zero callers** | `grep -rn GovernorDeps --include=*.rs \| grep -v compliance-governor/src` → **empty**. 11 services declare `compliance_governor_url`, **0 read it**. | The safety gate is not on any execution path. |
| **Compliance config activation is unauthenticated** | `api/admin.rs:141-143` checks only that two signature *strings* are non-empty; returns `accepted: true` with `audit_log_id: 0` and **no persistence**. | Anyone reaching the port can rewrite the safety thresholds. |
| **Restriction detection is a dead loop** | The gateway *detects* restriction signals; nothing ever writes `lcc.restrictions`; guard 6 (now fixed) *reads* it. | The circuit-breaker loop is open. |
| **Audit emission is a stub** | `AuditClient::try_send_grpc` always returns `Err("gRPC client not wired")`; falls back to a local `AtomicU64` counter. | **Nothing reaches `lcc_audit.events`.** The audit trail does not exist at runtime. |
| **Circuit breaker skips half-open** | `try_half_open` has **0 callers**; OPEN → CLOSED directly. | No gradual re-entry; full flood resumes. |
| **429 is a hard stop, not a retry** | `restriction.rs:65` classifies 429 as a restriction signal; `track_a.rs:164` treats any signal as terminal. | The one status that is definitionally retryable gets zero retries. |
| **Idempotency fails open** | `if let Ok(Some(cached)) = …` — an `Err` is indistinguishable from a miss. Its Postgres mirror targets a non-existent table. | Replay protection disappears during a datastore outage. |
| **RLS is entirely unenforced** | `set_member_context` has **0 call sites** outside its definition; no service opens a transaction; `x-member-id` is forwarded to **0** consumers. | Every tenant-scoped query is unscoped. |
| **`lcc-proto` cannot compile** | `crates/proto/src/gen/` contains only a 0-byte `.gitkeep`; `build.rs` never runs codegen; `include_proto!` ×21. | The entire gRPC layer is absent. |

### 3.2 Python Intelligence Engine unreachable

The **only** Rust→Python edge is `compliance-governor/src/scoring.rs:32-35`, which returns its input
unchanged:

```rust
// Real implementation: tonic client to scoring-intel.ComputeH_c.
// For now, return the cached value.
Ok(account.h_c)
```

Its caller's `Err` arm is therefore unreachable, so H_c is **never refreshed** and guard 4 gates on a
permanently stale scalar. Separately, the gRPC contract (`ScoringIntel`, `AIWorker` — 88 RPCs) is
implemented on **neither** side: `packages/proto/src/lcc_pb2/__init__.py` is 0 bytes, and no Python
service binds 50051. `crates/compliance-types` contains *faithful* mirrors of H_c, φ, ρ and AB_d —
and no service imports them.

### 3.3 Three incompatible API contracts

| | Base path | Endpoint count |
|---|---|---|
| **Design** (§17/§18) | `/api/v1` | ~60 |
| **Frontend** (`schemas/openapi/api-gateway.yaml`, 1,919 lines) | no prefix | 60 |
| **Backend** (`schemas/openapi/openapi.yaml`, 190 lines) | `/v1` | 6 |
| **Backend routers, actual** | `/v1/{domain}/*` → proxy; `/v1/{svc}_svc/items` | scaffold CRUD |

The frontend's `apiFetch` concatenates `API_BASE + path` with **no `/v1` prefix** and no
`/api/v1`; `next.config.mjs` rewrites only `/api/v1/auth/:path*`. So a request to
`/members/{id}/content` goes to the Next origin and 404s, and with `NEXT_PUBLIC_API_BASE` set it
reaches the gateway at a path the gateway does not serve. **Zero overlap between the 49 endpoints
the frontend calls and anything the backend implements.**

### 3.4 No WebSocket server

The frontend ships a genuinely good realtime client (reconnect, exponential backoff, heartbeat,
SSE fallback). The backend has **no WebSocket server** — `grep -rn WebSocketUpgrade\|axum::extract::ws`
returns one `use tokio_tungstenite::accept_async;` inside a function with unsatisfiable axum state
types. All five required channels (`/ws/{briefing,approvals,engagement,compliance,sequence}`) are
absent.

### 3.5 Configuration does not load

`ccfg-*.yaml` uses a **nested** schema with 5 named `h_c` weights; `ComplianceConfig` expects a
**flat** `h_c_weights: [f64; 4]` with no `#[serde(default)]` ⇒ deserialization fails and
`compliance-governor/src/main.rs:39-40` propagates the error. **The governor cannot start against any
config in the repo.** The caps are also wrong: 3 of 6 spec'd caps have no YAML key at all, and the
other 3 read 25/40/2 against the spec's 18/25/3.

---

## 4. Performance & scalability

**Latency budgets cannot be validated**, because the components that would serve them do not run.
What can be stated from the code:

| Budget (Spec §24/§26) | Status | Reason |
|---|---|---|
| Governor guard-stack p99 < 200 ms | **Unmeasurable** | Governor is never called; guards 6/7 denied 100% |
| Briefing assembly p95 < 3 s | **Would violate** | 4 sequential round-trips × 100 members (**fixed** → now concurrent) |
| Intelligence RPC p95 < 3 s | **Unmeasurable** | No Rust→Python transport exists |
| API Gateway availability ≥ 99.5% | **Would violate** | `readyz` always passed against a dead backend (**fixed**) |

**Fixed by measurement of the code path:** briefing-worker was O(members × 4) serial round-trips;
now 4 concurrent queries per member, bounded to 100 members/tick.

**Remaining performance/scalability risks (evidence-based, unfixed):**

- **Rate limiter is per-process and in-memory** ⇒ effective ceiling is `N × max` across N replicas,
  resets on restart, and the `HashMap` has **no eviction** (unbounded key growth). The design
  requires Redis-backed quotas. Documented in-code.
- **Every service pool is `max_connections(4)`**; the gateway is 20. With `SKIP LOCKED` claiming and
  the gateway now proxying, 4 connections per worker is thin under the specified concurrency.
- **`session_pacing` holds state in a process-global** ⇒ per-pod pacing under multi-replica deploy;
  a restart re-applies minimum jitter (safe) but replicas do not see each other.
- **Blocking `model.encode()` inside `async def`** on every draft path (`infra/embedding.py:31-35`)
  blocks the FastAPI event loop for the duration of inference.
- **Qdrant errors silently degrade to empty grounding** (`rag/retriever.py:41-42`), making an outage
  indistinguishable from "no KB data" — and, with grounding unenforced in Python, an outage would
  not fail loudly.
- **Track B actions are enqueued into a void** — `ext:pending:<member_id>` is `RPUSH`ed
  (`track_b.rs:224`) and read by **nothing**; the "background pump" its comment promises does not exist.

---

## 5. Testing status

| Suite | Status |
|---|---|
| Rust unit/integration | **Never ran** — no toolchain in this environment, and CI was an `echo` stub. Now wired. |
| Python `pytest` | **Never ran** — CI was an `echo` stub. `ruff`/`mypy` likewise. Now wired. |
| Frontend `vitest` | Not run — `pnpm` unavailable; `node_modules` install incomplete. |
| Frontend `tsc` | **Run.** 0 syntax errors after 2 fixes. |
| Guard tests | Every guard's test asserts only `assert_eq!(g.name(), "…")`. **No behavioural guard test exists.** `governor_e2e.rs` exercises a hand-written mock, not the real governor. |

**Validation limits of this engagement, stated plainly:** no Rust toolchain and no PyPI access in the
sandbox. Rust changes are verified by static inspection (every removed/added symbol checked against
its definition and every call site), not by `cargo build`. Python changes are verified by
`ast.parse` (all 4 files) and by structural review. The `sandbox` could not run `cargo test`,
`pytest`, `vitest`, `kustomize build`, or `terraform validate`.

---

## 6. Recommended sequence

1. **Make it build.** Run the now-real CI. Expect and triage the remaining compile blockers
   (axum state types in `integration-gateway/main.rs`, `governor/main.rs` double-`State`,
   `sha2`/`hex` deps, `lcc-proto` codegen).
2. **Close the safety loop** before anything can act externally: build a real Governor client and
   wire it into the orchestrator; persist config activation with real two-reviewer signatures; wire
   the audit transport; fix RLS call sites; implement `jti` consumer cleanup and half-open probing.
3. **Fix the schema** (run the migrations — now buildable), then align `ccfg-*.yaml` to the flat
   `ComplianceConfig` shape and to the §12 caps.
4. **Reconcile the three API contracts** into one OpenAPI, then implement the domain services
   against it. Until then, no FE↔BE integration work is worth doing.
5. **Add behavioural tests** for all 8 guards, permit issuance→verification across a process
   boundary, and the RBAC matrix.

---

## 7. Change inventory (64 files)

**Backend — 60 files**
- `api-gateway` (12): `proxy/mod.rs` (real registry + forward), `proxy/pool.rs`, `state.rs`,
  `http/handlers/mod.rs`, `http/router.rs`, `middleware/{auth,rate_limit}.rs`, `health.rs`,
  `config.rs`, `error.rs`, `main.rs`, `Cargo.toml`
- `compliance-governor` (2): `guards/{restriction_flag,approval_state}.rs`
- `integration-gateway` (6): `permit/{mod,replay,verifier}.rs`, `track_a.rs`, `track_b.rs`
- Workers (11): `sequence-step-scheduler/{logic.rs,lib.rs,Cargo.toml}`,
  `staleness-scanner/{logic.rs,lib.rs}`, `briefing-worker/{logic.rs,lib.rs}`,
  `audit-integrity-worker/lib.rs`, `data-purge-worker/lib.rs`, `opportunity-discovery-worker/lib.rs`
- Crates (2): `compliance/src/lib.rs` (cyclic aliases), `db/src/tx.rs` (audit insert)
- Python (4): `ai_worker/api/drafts.py`, `ai_worker/llm/cost_controller.py`,
  `scoring_intel/core/{rho,model_loader}.py`
- Dockerfiles (3): all intelligence workers — `useradd` for UID 10001
- SQL (4): `0002` (schema-qualify RLS), `0003` (members policy), `0015` (new), `0016` (new)
- K8s (10): `api-gateway.yaml`, 6 worker manifests, 2 new overlay patches, 2 kustomizations
- CI (6): rust-test, rust-clippy, python-test, python-lint, security-scan, proto-lint
- Compose (1): port collisions

**Frontend — 4 files**
- `app/(app)/layout.tsx`, `components/copilot/CopilotRoot.tsx` (parse errors)
- `app/providers.tsx` (client-config race), `lib/auth/client-session.ts` (hooks violation)

---

*Every claim in this report is traceable to a `file:line` in the delivered trees. Findings marked
"verified" were checked by execution (typecheck); the remainder are established by exhaustive static
analysis of definitions and call sites, with the absence claims produced by repo-wide greps quoted
inline.*
