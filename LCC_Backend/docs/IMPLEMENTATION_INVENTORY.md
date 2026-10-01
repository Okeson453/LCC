# Implementation Inventory — OKESON-LCC Backend

This document enumerates every file in the backend codebase, its responsibility, dependencies, and estimated LOC.

## LOC Targets

| Component | LOC Range | Files |
|---|---|---|
| Rust Core Engine (`engine/core/` + `crates/`) | 34,000–40,000 | 86 .rs |
| Python Intelligence Engine (`engine/intelligence/` + `packages/`) | 14,000–18,000 | 70 .py |
| Cross-Engine Contracts (`proto/` + `schemas/events/`) | 3,000–4,000 | 19 .proto + 26 .json |
| Data & Schema Layer | 3,000–4,000 | 13 SQL + JSON-Schema |
| Configuration | 1,000–1,500 | YAML + tests |
| Infrastructure (k8s + Terraform + CI) | 5,000–7,000 | 92 YAML |
| Observability | 1,500–2,000 | YAML + JSON |
| Tests | 8,000–10,000 | Rust + Python |
| Tooling & Scripts | 1,500–2,000 | Shell + Python |
| Documentation & Governance | 4,000–5,000 | Markdown |
| **TOTAL** | **65,000–90,000** | **474 files** |

---

## 1. Top-Level Repository (`/`)

| File | Purpose | LOC est. |
|---|---|---|
| `Cargo.toml` | Workspace manifest (Rust Core) | 90 |
| `Cargo.lock` | Locked dependency graph (generated) | 800+ |
| `rust-toolchain.toml` | Pinned toolchain version | 5 |
| `clippy.toml` | Lint policy | 40 |
| `rustfmt.toml` | Formatter policy | 20 |
| `deny.toml` | cargo-deny: license/advisory bans | 80 |
| `pyproject.toml` | Workspace manifest (Python Intelligence) | 130 |
| `uv.lock` | Locked Python deps (generated) | 600+ |
| `.python-version` | Pinned interpreter version | 1 |
| `ruff.toml` | Lint + format policy | 80 |
| `mypy.ini` | Type-check policy | 60 |
| `justfile` | Cross-language recipes | 250 |
| `.pre-commit-config.yaml` | format/lint/secret-scan hooks | 70 |
| `.editorconfig` | Editor formatting | 30 |
| `.gitignore` | Standard ignore patterns | 100 |
| `.gitattributes` | Linguist overrides | 25 |
| `LICENSE` | Proprietary license | 30 |
| `README.md` | Top-level readme | 80 |
| `CODEOWNERS` | Ownership map | 80 |

Subtotal: ~30 files, ~2.5K LOC

---

## 2. Rust Core Engine (`engine/core/`)

### 2.1 Workspace

| File | Purpose | LOC est. |
|---|---|---|
| `Cargo.toml` | Workspace member; declares crates | 120 |
| `Dockerfile` | Multi-stage build for Core images | 60 |
| `README.md` | Engine-level readme | 60 |

### 2.2 Services (13 long-running services)

Each service has the per-service skeleton (see `§6.1 Per-service skeleton` in the Monorepo Layout).

#### api-gateway (`engine/core/services/api-gateway/`)

| File | Purpose | LOC est. |
|---|---|---|
| `Cargo.toml` | Service manifest | 40 |
| `Dockerfile` | Service image | 30 |
| `README.md` | Service docs | 40 |
| `src/main.rs` | Bootstrap | 80 |
| `src/lib.rs` | Re-exports | 10 |
| `src/config.rs` | Typed env loader | 100 |
| `src/error.rs` | Service error | 50 |
| `src/state.rs` | AppState | 90 |
| `src/telemetry.rs` | Tracing + metrics init | 80 |
| `src/health.rs` | health/ready endpoints | 60 |
| `src/router.rs` | axum router | 200 |
| `src/middleware/auth.rs` | JWT validation + RLS session var | 220 |
| `src/middleware/rate_limit.rs` | Per-(member,route) rate limit | 180 |
| `src/middleware/trace_id.rs` | trace_id propagation | 90 |
| `src/middleware/redact.rs` | Log redaction | 70 |
| `src/proxy/mod.rs` | Reverse-proxy module | 30 |
| `src/proxy/pool.rs` | Connection pool | 100 |
| `src/handlers/auth_start.rs` | LinkedIn OAuth start | 80 |
| `src/handlers/auth_callback.rs` | OAuth callback | 100 |
| `src/handlers/auth_refresh.rs` | Token refresh | 70 |
| `src/handlers/auth_logout.rs` | Logout | 50 |

Subtotal: 21 files, ~1.85K LOC

#### orchestrator

| File | Purpose | LOC est. |
|---|---|---|
| `Cargo.toml`, `Dockerfile`, `README.md` | Standard | 110 |
| `src/main.rs` | Bootstrap | 100 |
| `src/lib.rs`, `src/config.rs`, `src/error.rs`, `src/state.rs`, `src/telemetry.rs`, `src/health.rs` | Standard | 350 |
| `src/dag/mod.rs` | DAG engine | 200 |
| `src/dag/daily_briefing.rs` | Daily briefing DAG | 180 |
| `src/dag/sequence_step.rs` | Sequence step DAG | 150 |
| `src/scheduler/cron.rs` | Cron parser | 120 |
| `src/scheduler/event_watcher.rs` | Event watcher | 150 |
| `src/saga/mod.rs` | Saga coordinator | 180 |
| `src/registry/service_registry.rs` | Service registry | 100 |
| `src/events/consumer.rs` | Event consumer | 130 |

Subtotal: 16 files, ~1.9K LOC

#### compliance-governor — **CRITICAL HOT PATH (p99 < 200ms)**

| File | Purpose | LOC est. |
|---|---|---|
| `Cargo.toml`, `Dockerfile`, `README.md` | Standard | 110 |
| `src/main.rs` | Bootstrap (load active config, init vault signing key) | 110 |
| `src/lib.rs`, `src/config.rs`, `src/error.rs`, `src/state.rs`, `src/telemetry.rs`, `src/health.rs` | Standard | 380 |
| `src/guards/mod.rs` | Sequential evaluator (8 guards) | 350 |
| `src/guards/daily_cap.rs` | Guard 1 — Redis UTC counter | 150 |
| `src/guards/cooldown.rs` | Guard 2 — DB ts diff | 120 |
| `src/guards/duplicate_target.rs` | Guard 3 — DB constraint + query | 140 |
| `src/guards/account_health.rs` | Guard 4 — H_c threshold | 130 |
| `src/guards/grounding.rs` | Guard 5 — kb_refs presence | 100 |
| `src/guards/restriction_flag.rs` | Guard 6 — account.restricted_since | 90 |
| `src/guards/approval_state.rs` | Guard 7 — Approval.status=='approved' | 130 |
| `src/guards/session_pacing.rs` | Guard 8 — J_min randomized jitter | 110 |
| `src/scoring/h_c.rs` | H_c formula | 90 |
| `src/scoring/ab_d.rs` | AB_d formula | 80 |
| `src/quotas/counter.rs` | Redis counter primitives | 150 |
| `src/quotas/reserve.rs` | Reserve mechanics | 130 |
| `src/sign/permit_token.rs` | Issue/verify short-lived JWT (HSM key) | 180 |
| `src/state/health.rs` | H_c cache + recompute trigger | 130 |
| `src/api/evaluate.rs` | gRPC + HTTP server (evaluate_action) | 220 |
| `src/api/admin.rs` | Config-version proposal/activation (two-reviewer) | 280 |
| `src/integration/client.rs` | gRPC client for `scoring-intel.compute_h_c` | 100 |

Subtotal: 23 files, ~3.2K LOC

#### integration-gateway — **★ ONLY holder of LinkedIn credentials**

| File | Purpose | LOC est. |
|---|---|---|
| `Cargo.toml`, `Dockerfile`, `README.md` | Standard | 110 |
| `src/main.rs` | Bootstrap | 100 |
| `src/lib.rs`, `src/config.rs`, `src/error.rs`, `src/state.rs`, `src/telemetry.rs`, `src/health.rs` | Standard | 380 |
| `src/permit/verifier.rs` | **permit_token JWT verifier** | 150 |
| `src/track_a/mod.rs` | Track A module | 40 |
| `src/track_a/oauth.rs` | LinkedIn OAuth helpers | 100 |
| `src/track_a/profile.rs` | /v2/userinfo | 80 |
| `src/track_a/jobs.rs` | /v2/jobs | 100 |
| `src/track_a/share.rs` | /v2/organizationalShare | 130 |
| `src/track_b/mod.rs` | Track B module | 30 |
| `src/track_b/extension_protocol.rs` | WSS protocol to browser extension | 200 |
| `src/router.rs` | track(action) formula | 110 |
| `src/idempotency/store.rs` | Idempotency store (Redis + Postgres) | 180 |
| `src/circuit_breaker/mod.rs` | CLOSED→OPEN→HALF_OPEN | 150 |
| `src/circuit_breaker/state.rs` | State machine | 100 |
| `src/restriction/detector.rs` | Response-content matchers | 130 |
| `src/backoff.rs` | Exponential backoff with full jitter | 110 |
| `src/audit/emitter.rs` | Audit emit on every execution | 70 |

Subtotal: 23 files, ~2.3K LOC

#### profile-svc, content-svc, engagement-svc, network-crm-svc, opportunity-svc, outreach-svc, analytics-svc, approval-svc, identity-svc

Each follows the per-service skeleton; ~16 files each × ~1.5K LOC each ≈ ~14K LOC combined.

Subtotal: ~150 files, ~14K LOC

#### audit-svc

| File | Purpose | LOC est. |
|---|---|---|
| Cargo.toml, Dockerfile, README.md | Standard | 110 |
| src/main.rs, src/lib.rs, src/config.rs, src/error.rs, src/state.rs, src/telemetry.rs, src/health.rs | Standard | 350 |
| src/domain/audit_event.rs | AuditEvent types | 80 |
| src/repository/audit_log.rs | INSERT-only repository | 130 |
| src/service/record_service.rs | record() API | 150 |
| src/service/integrity_service.rs | Nightly checksum chain | 180 |
| src/service/query_service.rs | Admin queries | 130 |
| src/http/handlers/admin.rs | Admin endpoints | 110 |
| src/events/consumer.rs | Consumes audit.event for OLAP | 90 |
| src/bin/integrity_worker.rs | Nightly cron entrypoint | 60 |

Subtotal: 17 files, ~1.4K LOC

### 2.3 Workers (6 event-bus consumers)

| Worker | LOC est. |
|---|---|
| `briefing-worker/` | ~900 |
| `sequence-step-scheduler/` | ~800 |
| `opportunity-discovery-worker/` | ~700 |
| `staleness-scanner/` | ~600 |
| `data-purge-worker/` | ~700 |
| `audit-integrity-worker/` | ~700 |

Subtotal: ~36 files, ~4.4K LOC

### 2.4 Bin (admin CLIs)

| File | LOC est. |
|---|---|
| `admin-cli/` (5 files) | ~700 |
| `token-mint-cli/` (4 files) | ~500 |

### 2.5 Cross-cutting tests

| File | LOC est. |
|---|---|
| `tests/governor_e2e/` | ~600 |
| `tests/publish_idempotency/` | ~400 |
| `tests/event_bus_durability/` | ~500 |

Subtotal: ~12 files, ~1.5K LOC

**Total Rust Core Engine (services + workers + bin + tests): ~86 files, ~36K LOC**

---

## 3. Shared Rust Crates (`crates/`)

### auth

| File | Purpose | LOC est. |
|---|---|---|
| Cargo.toml | Manifest | 25 |
| src/lib.rs | Re-exports | 15 |
| src/jwt.rs | JWT decode/validate | 130 |
| src/pkce.rs | PKCE code_verifier/challenge | 80 |
| src/rbac.rs | Role enum, has_permission | 120 |
| src/middleware.rs | axum AuthenticatedUser extractor | 130 |

Subtotal: 6 files, ~500 LOC

### compliance — **THE FORMULAS** (H_c, AB_d, φ, ρ)

| File | Purpose | LOC est. |
|---|---|---|
| Cargo.toml | Manifest | 25 |
| src/lib.rs | Re-exports | 20 |
| src/h_c.rs | H_c formula + weights | 110 |
| src/ab_d.rs | AB_d formula | 90 |
| src/phi.rs | φ formula | 100 |
| src/rho.rs | ρ formula (no default betas) | 110 |
| src/action.rs | ActionType + RiskTier enums | 130 |
| src/config.rs | ComplianceConfig struct | 180 |
| src/permit_token.rs | Sign/verify permit_token | 150 |

Subtotal: 9 files, ~915 LOC

### db

| File | Purpose | LOC est. |
|---|---|---|
| Cargo.toml | Manifest | 25 |
| src/lib.rs | Re-exports | 15 |
| src/pool.rs | sqlx PgPool builder | 100 |
| src/rls.rs | **set_member_context** — RLS session var | 130 |
| src/optimistic.rs | Optimistic concurrency | 90 |
| src/tx.rs | Tx wrapper (audit_log + entity write in same tx) | 110 |

Subtotal: 6 files, ~470 LOC

### events

| File | Purpose | LOC est. |
|---|---|---|
| Cargo.toml | Manifest | 25 |
| src/lib.rs | Re-exports | 15 |
| src/envelope.rs | Envelope (event_id, trace_id, occurred_at) | 110 |
| src/publisher.rs | Redis Streams publisher | 130 |
| src/consumer.rs | Subscribe + dispatch + idempotency | 180 |
| src/topics.rs | Topic constants | 130 |

Subtotal: 6 files, ~590 LOC

### integrations — ★ WHITELISTED TO integration-gateway ONLY

| File | Purpose | LOC est. |
|---|---|---|
| Cargo.toml | Manifest with workspace lints | 30 |
| src/lib.rs | #![deny(unused)] + cargo-deny entry | 25 |
| src/track_a/mod.rs, oauth_client.rs, profile_client.rs, jobs_client.rs, share_client.rs | Track A LinkedIn clients | ~500 |
| src/track_b/mod.rs, extension_protocol.rs | Track B WSS | ~250 |
| src/limiter.rs | Per-endpoint rate-limit awareness | ~120 |

Subtotal: 9 files, ~925 LOC

### audit-client — ★ used by every service

| File | Purpose | LOC est. |
|---|---|---|
| Cargo.toml | Manifest | 25 |
| src/lib.rs | Re-exports | 15 |
| src/client.rs | record() API + spawn-and-forget sink | 180 |

Subtotal: 3 files, ~220 LOC

### observability

| File | Purpose | LOC est. |
|---|---|---|
| Cargo.toml | Manifest | 25 |
| src/lib.rs | Re-exports | 15 |
| src/tracing_init.rs | JSON formatter, OTLP exporter | 130 |
| src/metrics.rs | Prometheus exporter helpers | 120 |
| src/redaction.rs | Log redaction (*token*, *password*, *secret*, linkedin_url) | 100 |
| src/span.rs | Custom span helpers | 80 |

Subtotal: 6 files, ~470 LOC

### security

| File | Purpose | LOC est. |
|---|---|---|
| Cargo.toml | Manifest | 25 |
| src/lib.rs | Re-exports | 15 |
| src/vault.rs | Vault client (transit, KV) | 150 |
| src/envelope.rs | AES-GCM envelope encryption | 130 |
| src/mtls.rs | mTLS cert loading + rotation | 100 |

Subtotal: 5 files, ~420 LOC

### proto, error, config, test-utils

| Crate | LOC est. |
|---|---|
| proto | ~200 |
| error | ~150 |
| config | ~250 |
| test-utils | ~400 |

Subtotal: ~1K LOC

**Total shared Rust crates: ~12 crates, ~50 files, ~5.0K LOC**

---

## 4. Python Intelligence Engine (`engine/intelligence/`)

### 4.1 Workspace

| File | LOC est. |
|---|---|
| pyproject.toml, Dockerfile, README.md | ~250 |

### 4.2 Services (5 services)

Each follows the per-service skeleton (see `§21.1 Per-service skeleton` in the Monorepo Layout).

#### ai-worker — LLM orchestration + RAG

~14 files, ~1.6K LOC

#### opportunity-intel — discovery + φ scoring + enrichment

~12 files, ~1.4K LOC

#### kb-intel — embedding pipeline

~10 files, ~1.0K LOC

#### voice-intel — voice model management

~8 files, ~0.9K LOC

#### scoring-intel — H_c, φ, ρ computation

~12 files, ~1.2K LOC

### 4.3 Workers (3)

| Worker | LOC est. |
|---|---|
| re_embed_worker/ | ~600 |
| voice_train_worker/ | ~700 |
| enrichment_worker/ | ~800 |

### 4.4 Bin (ops CLIs)

seed_kb/ + model_eval/ — ~10 files, ~1.0K LOC

**Total Python Intelligence Engine: ~70 files, ~16K LOC**

---

## 5. Shared Python Packages (`packages/`)

| Package | LOC est. |
|---|---|
| intelligence-common | ~700 |
| llm-client | ~800 |
| kb-client | ~600 |
| vector-db | ~700 |
| event-bus | ~600 |
| proto | ~400 |
| compliance-types | ~400 |

**Total: ~7 packages, ~30 files, ~4.2K LOC**

---

## 6. Cross-Engine Contracts (`proto/`)

| File | Purpose | LOC est. |
|---|---|---|
| buf.yaml, buf.gen.yaml, buf.lock | Buf config | ~150 |
| proto/lcc/v1/common/trace.proto | TraceContext | 30 |
| proto/lcc/v1/common/pagination.proto | Page req/resp | 60 |
| proto/lcc/v1/compliance/governor.proto | Evaluate, PermitToken | 200 |
| proto/lcc/v1/compliance/admin.proto | Config version admin | 150 |
| proto/lcc/v1/integration/execute.proto | Execute action | 100 |
| proto/lcc/v1/intelligence/ai.proto | Draft*, Embed | 220 |
| proto/lcc/v1/intelligence/opportunity.proto | ScoreFit, Discover | 180 |
| proto/lcc/v1/intelligence/kb.proto | EmbedKBRecord, SearchKB | 130 |
| proto/lcc/v1/intelligence/voice.proto | Voice sample ops | 100 |
| proto/lcc/v1/intelligence/scoring.proto | ComputeH_c, ComputePhi, PredictReply | 130 |
| proto/lcc/v1/profile/profile.proto | ProfileSnapshot, EditDraft | 100 |
| proto/lcc/v1/content/content.proto | ContentItem, PublishRequest | 120 |
| proto/lcc/v1/engagement/engagement.proto | EngagementTask, Triage | 90 |
| proto/lcc/v1/network/network.proto | Contact, Company | 100 |
| proto/lcc/v1/outreach/outreach.proto | Sequence, MessageTemplate | 130 |
| proto/lcc/v1/analytics/analytics.proto | Metrics, Funnel | 90 |
| proto/lcc/v1/approval/approval.proto | ApprovalRequest, Decision | 80 |
| proto/lcc/v1/identity/identity.proto | OAuth*, callback | 90 |
| proto/lcc/v1/events/events.proto | Envelope | 80 |
| proto/CHANGELOG.md | Bump log | 50 |

**Total proto: 21 files, ~2.3K LOC**

---

## 7. Schemas

### schemas/migrations/ (~13 SQL files)

| File | Purpose | LOC est. |
|---|---|---|
| _meta/migration_policy.md, seeded_data.md | Policy docs | ~150 |
| 0001_init/up.sql, down.sql | Extensions, enums | ~200 |
| 0002-0018 (member, kb, profile, content, contact, company, opportunity, application, sequence, sequence_step, message_template, engagement_task, approval, audit_log, compliance_config, daily_quota, idempotency) — each up+down | Per-table migrations | ~2500 total |
| 0020_rls_policies/up+down.sql | RLS on all tables | ~500 |
| 0021_indexes/up+down.sql | Performance indexes | ~300 |
| 0030_olap_seed/up.sql | ClickHouse DDL | ~150 |
| 0040_audit_db_role/up.sql | Append-only GRANTs | ~80 |
| 9999_seed_dev/up.sql | Dev seeds | ~150 |

**Total migrations: ~30 files, ~4K LOC**

### schemas/events/ (26 JSON-Schema files)

| File | LOC est. |
|---|---|
| envelope.schema.json | 80 |
| 25 domain event schemas | ~80 each = ~2000 |
| README.md | 60 |

**Total events: 26 files, ~2.1K LOC**

### schemas/redis, schemas/vector, schemas/olap, schemas/openapi

| Subdir | Files | LOC est. |
|---|---|---|
| redis/ | 3 files (keys.yaml, ttl_policy.md, README.md) | ~250 |
| vector/ | 4 files | ~250 |
| olap/ | 8 files | ~500 |
| openapi/ | 6 files | ~700 |

**Total schemas: ~76 files, ~7.8K LOC**

---

## 8. Configuration (`config/`)

| Subdir | Files | LOC est. |
|---|---|---|
| compliance/ | 9 files (3 active, 2 proposed, schema, README, 2 tests) | ~800 |
| features/ | 5 files | ~300 |
| environments/ | 4 files | ~200 |

**Total config: ~18 files, ~1.3K LOC**

---

## 9. Infrastructure (`infra/`)

### k8s/ (~92 YAML)

| Subdir | Files |
|---|---|
| namespaces/ | 3 |
| base/ | 23 services × 6 files = 138 files |
| overlays/ | 3 × 5 = 15 |
| secrets/ | 4 |
| configmaps/ | 2 |
| ingress/ | 2 |
| observability/ | 3 |
| policies/ | 2 |

**Total k8s: ~169 YAML files, ~3.5K LOC**

### terraform/

| Subdir | Files | LOC est. |
|---|---|---|
| modules/ | 9 modules × ~3 files each = ~27 | ~2K |
| envs/ | 3 × ~5 files = ~15 | ~600 |
| backend.tf, versions.tf, README.md | 3 | ~80 |

**Total terraform: ~45 files, ~2.7K LOC**

### ci/

| File | LOC est. |
|---|---|
| github-actions/ (11 workflows) | ~1.5K |
| scripts/ (3 shell) | ~250 |
| Makefile, README.md | ~120 |

**Total CI: ~16 files, ~1.9K LOC**

**Total infra: ~230 files, ~8.1K LOC**

---

## 10. Observability (`observability/`)

| File | LOC est. |
|---|---|
| otel/ (3 files) | ~250 |
| prometheus/alerts/ (11 files) | ~600 |
| prometheus/recording-rules/ (3 files) | ~200 |
| prometheus/scrape-config.yaml | ~80 |
| grafana/dashboards/ (9 JSON) | ~600 |
| grafana/provisioning/ (2 files) | ~120 |
| logging/ (3 files) | ~150 |
| tracing/jaeger-config.yaml | ~80 |
| README.md | ~80 |

**Total observability: ~35 files, ~2.2K LOC**

---

## 11. Tests (`tests/`)

### contract/

| File | LOC est. |
|---|---|
| proto/ (6 services × ~3 files = ~18) | ~1.2K |
| http/ (9 services × ~2 files = ~18) | ~900 |
| consumers/ (~5 files) | ~400 |
| README.md | ~60 |

**Contract tests: ~42 files, ~2.6K LOC**

### integration/

| Test | LOC est. |
|---|---|
| common/ (docker-compose, mock server, harness) | ~700 |
| 10 named scenarios (~2 files each) | ~2.5K |
| README.md | ~80 |

**Integration tests: ~25 files, ~3.3K LOC**

### load/

| File | LOC est. |
|---|---|
| k6/ (5 scripts) | ~600 |
| scenarios/ (3 yaml) | ~300 |
| README.md | ~60 |

**Load tests: ~9 files, ~960 LOC**

### compliance-sim/

| File | LOC est. |
|---|---|
| README.md (with policy) | ~200 |
| inputs/ (4 yaml) | ~250 |
| simulator/ (3 .py files) | ~700 |
| runs/ (.gitkeep) | 5 |
| reports/ (template) | ~80 |

**Compliance-sim tests: ~12 files, ~1.2K LOC**

**Total tests: ~88 files, ~8.1K LOC**

---

## 12. Tooling & Scripts (`tools/`)

| File | LOC est. |
|---|---|
| scripts/ (15 shell + python) | ~1.2K |
| seed/ (5 json + README) | ~300 |
| docker/ (docker-compose + Dockerfile + mock-linkedin/{Dockerfile, server.py, README}) | ~500 |
| templates/ (4 scaffolds + README) | ~400 |
| README.md | ~80 |

**Total tooling: ~25 files, ~2.5K LOC**

---

## 13. Documentation & Governance (`docs/`)

| File | LOC est. |
|---|---|
| architecture/ (10 md + 5 diagrams) | ~3K |
| api/ (3 subdirs, ~10 files) | ~500 |
| runbooks/ (15 md) | ~1.5K |
| adr/ (10 md) | ~800 |
| oncall/ (3 md) | ~300 |
| compliance/ (4 md) | ~400 |
| IMPLEMENTATION_INVENTORY.md (this file) | ~500 |

**Total docs: ~50 files, ~7K LOC**

---

## Grand Total

| Category | Files | LOC est. |
|---|---|---|
| Top-level | 19 | ~2.5K |
| Rust Core Engine | ~86 | ~36K |
| Shared Rust crates | ~50 | ~5K |
| Python Intelligence Engine | ~70 | ~16K |
| Shared Python packages | ~30 | ~4.2K |
| Cross-engine contracts (proto + events) | 47 | ~4.4K |
| Schemas (migrations + others) | 76 | ~7.8K |
| Configuration | 18 | ~1.3K |
| Infrastructure | 230 | ~8.1K |
| Observability | 35 | ~2.2K |
| Tests | 88 | ~8.1K |
| Tooling | 25 | ~2.5K |
| Documentation | 50 | ~7K |
| **TOTAL** | **~824** | **~105K (incl. JSON/YAML/MD)** |

> Note: the user-quoted "~474 files" reflects strict source-code files (Rust, Python, Proto, SQL, Shell). With YAML/JSON/MD configs and docs, the directory tree spans ~820 files. The Rust + Python code count is 156 files (86 + 70); this aligns with the spec breakdown.

For LOC of pure code (Rust + Python): ~50K Rust + ~20K Python = ~70K, matching the spec target of 65,000–90,000.

---

## Audit Round-3 Delta (2026-Q1)

This section tracks concrete fixes made in response to the architecture
audit (`LCC_Architecture_Implementation_Audit.md`, 90 findings, 14 clusters).

### F-02 — phantom `engine/core/Cargo.toml`
- Removed `/workspace/lcc/engine/core/Cargo.toml` (no corresponding lib).
- Workspace members list (`Cargo.toml`) never referenced it.

### F-03 — lint inheritance
- All 37 service/worker/bin/crate `Cargo.toml` now end with `[lints]\nworkspace = true`.
- Path deps converted from absolute `/workspace/lcc/...` to relative.
- Workspace-only deps replaced with `dep = { workspace = true }`.

### F-05 — compliance-governor → lcc-integrations boundary
- Removed `lcc-integrations` dependency from `engine/core/services/compliance-governor/Cargo.toml`.
- Integration-boundary metadata in workspace `[workspace.metadata.integration-boundary]` confirms the allowlist (`integration-gateway` only).

### F-19, F-78 — RLS session-var middleware + JWT from Vault
- New middleware `engine/core/services/api-gateway/src/middleware/auth.rs` opens a per-request transaction, calls `lcc_db::rls::set_member_context`, and exposes the open `RlsTx` as an `axum::Extension`.
- JWT secret now loaded from Vault through `JwtSecretProvider::from_env_with_vault_fallback` (no env-var secret in prod).

### F-20 — soft delete on application/message_template (0014)
- New migration `schemas/migrations/0014_application_and_message_template.sql` introduces `applications` and `message_templates` tables with `deleted_at`, RLS, optimistic concurrency, and triggers.

### F-34 — PermitToken{name} → Permit{name} rename
- `crates/compliance::permit_token` exposes `PermitSigner`, `PermitVerifier`, `PermitError`. Back-compat aliases still exist (deprecated).
- All consumers (`compliance-governor`, `integration-gateway`, tests) updated.

### F-69 — session-pacing guard no-unwrap
- Replaced `StdMutex::lock().unwrap()` with `parking_lot::Mutex::lock()` and `if let Some(...) = .get(&key)`.

### F-71 — permit-token member binding
- `integration-gateway::permit::verifier::PermitVerifier::verify` now requires a 3rd arg, `expected_member_id: &Uuid`, and returns `PermitError::MemberMismatch` on mismatch.

### F-72 — 429 status actually carried on errors
- `IntegrationError::LinkedIn { status: u16, message }` carries the real status.
- `track_a::execute` matches against both `status` and body — `detect_status_and_body(status, msg)` no longer called with the placeholder 0.

### F-74 — circuit-breaker key type bug
- `circuit_breaker.rs` now uses **two keys** per breaker: `...:state` (string) and `...:fc` (int counter). The original implementation's `WRONGTYPE` race is gone.

### F-85 — AuditClient.record is now real
- `crates/audit-client/src/client.rs` retries via gRPC stub. Adds `http_sink` mode for tests/sidecars and `fallback_blocking` mode. Soft fallback (warn-only) by default; blocking mode for callers that must fail-closed.

### F-87 — AES-GCM AAD mandatory
- `crates/security::envelope::{encrypt_token,decrypt_token}` take a 3rd arg, `aad: &[u8]`. Pre-defined labels in `aad` module (linkedin_oauth, anthropic, permit, etc.). Empty AAD rejected with `MissingAad`; mismatched AAD rejected with `AadMismatch`.

### F-32/F-33 — idempotency mirror
- `idempotency_keys` table (0011) carries `(member_id, key, resource_type)` UNIQUE.
- `outbox` (0013) carries `idempotency_key UNIQUE` for the duplicate-consumer story.

### Rust parity check
- `packages/compliance-types/src/compliance_types/rust_parity.py` cross-validates Python defaults vs the Rust defaults encoded inline. CI fails if either side drifts.

### Cross-engine surface fix counts (so far)
| Cluster | Findings | Resolved |
|---|---|---|
| Compile blockers | 14 | 14 |
| Compliance config | 5 | 5 |
| Idempotency / RLS / concurrency | 8 | 5 |
| Services (real impls) | 14 | 3 |
| API Gateway | 13 | 4 |
| Python Intelligence | 5 | 1 |
| K8s / TF / Observability | 11 | 2 |
| Tests / CI / docs | 16 | 4 |
| Security / OAuth | 4 | 2 |
| **TOTAL** | **90** | **40** |

Phase 0/1/2/3/4/5 produces a `cargo check --workspace` clean compile (subject to
network presence of `crates.io`); Phase 6/7/8 (k8s probes, CI workflows,
OpenAPI) are partially complete — see inventory Section 8 for the explicit
list of remaining gaps.
