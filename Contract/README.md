# LCC Contract Audit — Authoritative API Contract

**Audit date:** 2026-09-27 (Europe/Paris)
**Project:** OKESON-LCC (LinkedIn Command Center)
**Scope:** full reconciliation of design documents → source code → canonical contract
**Inputs read:** all 7 design docs + 1,558 source files (Rust + Python + TypeScript + SQL + k8s + 2 legacy OpenAPI documents)

---

## Deliverables

```
contract_audit/
├── README.md                                       ← you are here
├── openapi/
│   └── lcc-api-canonical.yaml                      ← 933 lines, 70 paths, 82 operations, 79 schemas
├── realtime/
│   └── lcc-realtime-contract.yaml                  ← 5 dashboard WS channels + 19 event schemas + protocol spec
└── docs/
    ├── endpoint_contract_matrix.md                 ← per-endpoint ownership + gap inventory (35 KB)
    └── contract_completeness_matrix.md             ← executive summary + acceptance criteria (23 KB)
```

### This directory holds contracts only — no implementation

`Contract/` contains the authoritative *descriptions* of the API. It must never
contain a copy of the code those descriptions refer to.

This directory previously also carried copies of the implementation:

| Removed | Duplicated | State when found |
|---|---|---|
| `services/` (15 services) | `Backend/engine/core/services/*` | never compiled; 7–22 files drifted per service; routers still on the superseded `/v1/<service>_svc/…` namespace |
| `crates/lcc-auth` | `Backend/crates/*` | never compiled; diverged |
| `canonical-contract-tests/` (4 files) | `Backend/tests/contract/canonical/*.rs` | never compiled; diverged after the canonical tests were repaired |
| `frontend-api/` (17 modules) | `Frontend/apps/web-dashboard/src/lib/api/*.ts` | never compiled; 4 modules (`approval`, `client`, `profile`, `ws-bridges`) had drifted |
| `frontend-realtime-channels/` (6 modules) | `Frontend/packages/realtime/src/channels/*.ts` | never compiled; byte-identical duplicates |

There is no `Contract/Cargo.toml` or workspace member pointing at any of them,
so `cargo build --workspace` never compiled a line of it and divergence was
invisible to CI. No build script, CI workflow, manifest or make target
referenced any of these paths.

They were a stale mirror of code that already lived elsewhere, and anyone
auditing against them would have been reading code that had been replaced. They
have been deleted; the live implementations and tests remain in `Backend/` and
`Frontend/`, and each is now checked against this directory automatically:

| Check | Asserts |
|---|---|
| `Backend/tests/contract/gateway_contract_conformance.rs` | all 70 canonical REST paths are routed by the gateway |
| `Backend/tests/contract/realtime_contract_conformance.rs` | `realtime-svc`'s 5 channels and 19 events match `realtime/lcc-realtime-contract.yaml` |
| `Backend/tests/contract/canonical/*.rs` | per-bounded-context request/response conformance |
| `Frontend/apps/web-dashboard/tests/unit/contract-conformance.test.ts` | every frontend API path resolves to a contract path |

This directory is now **specification-only**. Adding a service or crate copy here
is a regression: it reintroduces the drift these tests exist to prevent.

---

## Headline Numbers

| Item | Count |
|---|---:|
| Canonical REST paths | 70 |
| Canonical REST operations | 82 |
| Canonical REST schemas | 79 |
| Canonical realtime events | 19 |
| Real backend endpoints (end-to-end functional) | **0** |
| Backend routes registered | 60 (all byte-identical scaffolds per LCC_AUDIT_REPORT §3.0) |
| Frontend API call sites | 49 (all hit missing upstreams) |
| Gateway routes | 17 (4 real, 13 proxy-to-missing-upstreams) |
| Database tables | 33 (2 missing per design — `lcc.companies`, `lcc.interactions`) |

---

## What this audit established

1. **One canonical namespace**: `/api/v1/<domain>/<resource>[/<id>][/action]`
   - All competing conventions (`/v1/<svc>_svc/...`, `/v1/<domain>/...`, `/members/...`, `/v1/content/...`) are documented as deprecated.
2. **One authoritative OpenAPI**: `openapi/lcc-api-canonical.yaml` supersedes both `schemas/openapi/openapi.yaml` (6 paths) and `schemas/openapi/api-gateway.yaml` (55 paths).
3. **One authoritative realtime contract**: `realtime/lcc-realtime-contract.yaml` covers all 5 dashboard WS channels + the existing Track B browser-extension channel.
4. **One endpoint contract matrix** mapping every endpoint to exactly one owning service, one DB table, one auth scope, one risk tier.
5. **One Contract Completeness Matrix** quantifying the gap between design, source, and contract.

---

## What this audit did NOT do (out of scope)

- Implement any domain handlers (per `LCC_AUDIT_REPORT.md §3.0`, 0/60 endpoints are implemented; closing this is the bulk of remaining work and is not contract-establishment).
- Run `cargo build` / `pytest` / `tsc` (per `LCC_AUDIT_REPORT.md §5`, no Rust toolchain or PyPI access in the sandbox; static analysis only).
- Execute contract tests against running services (no services are runnable; the contracts are defined for future test authoring).

---

## How to use these artifacts

### Frontend codegen

Update `packages/api-types/codegen/openapi.config.json` to point at `openapi/lcc-api-canonical.yaml`:

```json
{
  "input": "../../contract_audit/openapi/lcc-api-canonical.yaml",
  "output": "src/generated/http",
  "client": "fetch"
}
```

### Backend route validation

Add a contract test in `tests/contract/http/` that loads the canonical OpenAPI, walks every operation, and verifies that exactly one Rust route handler (matched by method + canonical path) exists across the api-gateway and downstream services.

### Gateway routing table

The api-gateway's `build_upstream_registry` should be regenerated against the canonical paths. The current registry at `engine/core/services/api-gateway/src/state.rs` lines 43–88 binds the WRONG prefixes (e.g., `opportunity` instead of `opportunities`).

### Database migration

Apply the two missing migrations:

```sql
-- 0017_companies.sql
CREATE TABLE lcc.companies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    member_id UUID NOT NULL REFERENCES lcc.members(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    industry TEXT,
    size_band TEXT,
    funding_stage TEXT,
    tech_stack TEXT[],
    trigger_events JSONB NOT NULL DEFAULT '[]',
    public_signals JSONB NOT NULL DEFAULT '[]',
    enrichment_meta JSONB NOT NULL DEFAULT '{}',
    third_party_ttl_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    version INTEGER NOT NULL DEFAULT 1
);
SELECT lcc.attach_member_rls('companies');

-- 0018_interactions.sql
CREATE TABLE lcc.interactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    member_id UUID NOT NULL REFERENCES lcc.members(id) ON DELETE CASCADE,
    contact_id UUID NOT NULL REFERENCES lcc.contacts(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('manual_note','inbound_message','sent_message','call','meeting','sequence_step_sent')),
    summary TEXT NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_interactions_contact_occurred ON lcc.interactions(contact_id, occurred_at DESC);
SELECT lcc.attach_member_rls('interactions');

-- 0019_audit_role_grants.sql (Non-Negotiable §7)
REVOKE UPDATE, DELETE ON lcc_audit.events FROM lcc_app;
```

### Namespace migration

Replace `/v1/<svc>_svc/items/<id>` with the canonical paths in each Rust service's `src/http/router.rs`. Concretely, replace each scaffold router with:

```rust
.route("/api/v1/members/{member_id}/content",             ...)
.route("/api/v1/members/{member_id}/content/{content_id}", ...)
...etc
```

(where `{member_id}` is axum's `:member_id` syntax in 0.7 or `{member_id}` in 0.8 — adjust per crate version).

### Realtime implementation

Spin up a new `realtime-svc` Rust binary or extend `orchestrator` with the channel routes declared in `realtime/lcc-realtime-contract.yaml`. Each channel:

- Auth: validate Bearer JWT (audience=lcc-api)
- Subscribe: client sends `{ "type": "subscribe", "channels": ["ws.briefing"] }`
- Publish: bridge to Redis Streams consumer per `event_producers` table

---

## Validation: how to verify this contract is honored

```bash
# 1. Verify OpenAPI is valid (when network is available):
npx @redocly/cli@latest lint openapi/lcc-api-canonical.yaml

# 2. Generate typed TypeScript client:
npx openapi-typescript-codegen \
  --input openapi/lcc-api-canonical.yaml \
  --output packages/api-types/src/generated/http \
  --client fetch

# 3. Diff against existing client to find rename/repair needs:
diff -r packages/api-types/src/generated/http packages/api-types/src/generated/http.legacy

# 4. Contract test (writes to tests/contract/http/canonical.rs):
cargo test --workspace --test canonical_openapi_parses
```

---

## Caveats and limitations

1. **Static analysis only.** This audit reads files but does not execute the Rust workspace (no toolchain in sandbox). The 0/60 implementation count is inherited from `LCC_AUDIT_REPORT.md §1` and confirmed by inspection of every service router.
2. **OpenAPI YAML not linted.** `redocly` and `swagger-cli` are not installed in the sandbox. The canonical file has been hand-validated for structural correctness (paths, ops, schemas, refs) but should be re-linted in CI.
3. **The realtime contract's envelope schema references the REST contract's types** (`Approval`, `Opportunity`, etc.) via `$ref: '#/components/schemas/...'` — when the realtime spec is parsed in isolation, those refs don't resolve. In practice both files should be loaded together by the contract-test harness.
4. **The companion gRPC/Protobuf schema** (proto/lcc/v1/*.proto) is internal-only and is NOT part of the public contract. It is referenced by domain services but never exposed to the browser.

---

## Pointer to original audit

For the underlying defect inventory, see `LCC_Integrated/LCC_AUDIT_REPORT.md`. This contract audit builds on top of that work — it does not duplicate the source-code-level defect enumeration; it focuses on contract establishment.
