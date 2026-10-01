# OKESON-LCC — LinkedIn Manager Backend

Production-grade backend for OKESON-LCC, a LinkedIn manager platform with
LLM-assisted content drafting, compliance-governed outbound actions,
opportunity discovery, and a single multi-tenant database.

## Two-Engine Architecture

- **Rust Core Engine** (`engine/core/`) — transactional, safety-critical:
  Compliance Governor, Integration Gateway, all domain services, audit log.
- **Python Intelligence Engine** (`engine/intelligence/`) — LLM/ML/scoring:
  ai-worker (RAG + drafts), opportunity-intel (φ), kb-intel, voice-intel,
  scoring-intel (ρ/ab_d/h_c), and async workers.
- Communication is **only** via gRPC contracts (`proto/lcc/v1/*.proto`) or
  the event bus (Redis Streams in Phase 1-2; Kafka in Phase 3+).
- Enforced by `infra/ci/scripts/check_boundaries.sh` (CI-gated).

## Non-negotiable invariants

1. **Single Compliance Governor** issues permit-tokens; integration-gateway
   verifies them (HMAC, ≤60s TTL, audience="integration-gateway").
2. **Integration Gateway** is the sole holder of LinkedIn credentials.
3. **Idempotency** by construction — every write-side call carries an
   idempotency key stored in Redis + Postgres.
4. **Audit log in the same DB transaction** as the entity write.
5. **RLS** per-member isolation keyed on `app.current_member_id`.
6. **No unwrap/expect/panic/todo** in Rust; clippy `-D warnings`.

## SLOs

| Component | Target |
|-----------|--------|
| Compliance Governor evaluate p99 | < 200ms |
| API Gateway non-LLM p99 | < 300ms |
| Permit-token verify p99 | < 50ms |
| OAuth token refresh p95 | < 1s |
| Briefing generation p95 | < 3s |
| AI worker draft p95 | < 3s |
| API availability monthly | ≥ 99.5% |

## Repository layout

```
lcc/
├── Cargo.toml                  # Rust workspace manifest
├── pyproject.toml              # Python Intelligence workspace
├── proto/                      # gRPC contracts (lcc.v1)
├── crates/                     # shared Rust crates (12)
├── engine/
│   ├── core/                   # Rust Core Engine
│   │   ├── services/           # 13 long-running services
│   │   ├── workers/            # 6 background workers
│   │   └── bin/                # 5 admin binaries
│   └── intelligence/           # Python Intelligence Engine
│       ├── services/           # 5 services
│       ├── workers/            # 3 async workers
│       └── bin/                # 2 admin tools
├── packages/                   # 8 shared Python packages
├── schemas/                    # SQL migrations + JSON-Schema + OpenAPI
├── config/                     # ccfg-*.yaml + features + environments
├── infra/                      # k8s, terraform, docker, ci
├── observability/              # otel, prometheus, grafana
├── tests/                      # contract, integration, load, compliance-sim
├── tools/                      # scripts, seed, templates
└── docs/                       # architecture, runbooks, adr, compliance
```

## Quick start (local dev)

```bash
# 1. Bring up the stack.
just bootstrap

# 2. Apply migrations + seed.
just migrate
just seed

# 3. Smoke tests.
just test

# 4. Tear down.
just teardown
```

## Compliance config

Compliance configurations live in `config/compliance/ccfg-*.yaml`.
Per Backend Design Concept §47, every config activation requires
**two compliance reviewers** + 1 activator. See
`docs/compliance/change-management.md`.

The active version is recorded in the `compliance_config_versions` table;
the Compliance Governor reads it on every evaluate call.

## Boundary checks (CI-gated)

```bash
# Reject any cross-engine import.
infra/ci/scripts/check_boundaries.sh
```

This script is run in every CI pipeline and fails the build on any
boundary violation.

## Documentation

- `docs/architecture/` — overview, components, data flow
- `docs/api/` — OpenAPI reference (canonical: `schemas/openapi/openapi.yaml`)
- `docs/runbooks/` — operational runbooks
- `docs/adr/` — Architecture Decision Records
- `docs/compliance/` — audit retention, change management, SLOs
- `docs/oncall/` — rotation + escalation

## License

Proprietary.
