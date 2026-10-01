# LCC Endpoint Contract Matrix (v1.0)

**Source of truth:** `/workspace/contract_audit/openapi/lcc-api-canonical.yaml`
**Cross-references:**
- Design: `LinkedIn_Manager_Backend_Design_Concept.md §17–§18`
- Frontend OpenAPI: `LCC_Frontend/lcc/schemas/openapi/api-gateway.yaml` (55 paths, 67 ops)
- Backend OpenAPI: `LCC_Backend/schemas/openapi/openapi.yaml` (6 paths, 5 ops)
- Backend routers: `LCC_Backend/engine/core/services/*/src/http/router.rs`
- Gateway: `LCC_Backend/engine/core/services/api-gateway/src/http/router.rs`
- DB: `LCC_Backend/schemas/migrations/*.sql`

**Legend (matrix columns):**
- **Status**: ✅ = design-required, present in canonical; ⚠️ = design-required, partially present (gap noted); ❌ = design-required, missing; 🔵 = frontend/ops-only, NOT in design (orphan)
- **Gateway** column shows the *current* gateway route registration.
- **Backend route** column shows the *current* backend service route registration.
- **DB** column shows the *current* migration-defined table.
- **Implementation** column shows whether a real handler exists or only a scaffold.

---

## A. INFRA (3 paths, 3 ops)

| Endpoint ID | Method | Canonical Path | FE calls | Gateway | Backend route | DB | Implementation |
|---|---|---|---|---|---|---|---|
| `infra.healthz` | GET | `/api/v1/healthz` | ✅ typed | ✅ `/healthz` | ✅ `/healthz` (gateway itself) | n/a | ✅ real per-upstream probe |
| `infra.readyz` | GET | `/api/v1/readyz` | ✅ typed | ✅ `/readyz` | ✅ `/readyz` (gateway itself) | n/a | ✅ real per-upstream probe |
| `infra.metrics` | GET | `/api/v1/metrics` | (Prometheus scrape) | ✅ `/metrics` (gateway) | ✅ (gateway) | n/a | ✅ real |

## B. AUTH & MEMBERS (Identity Svc) (8 paths, 11 ops)

| Endpoint ID | Method | Canonical Path | FE calls | Gateway | Backend route | DB | Implementation |
|---|---|---|---|---|---|---|---|
| `auth.linkedin_start` | GET | `/api/v1/auth/linkedin/start` | ✅ | ✅ `/v1/auth/start` | ✅ `handlers::auth_start` | `lcc.members`, `lcc.oauth_tokens` | ✅ |
| `auth.linkedin_callback` | GET | `/api/v1/auth/linkedin/callback` | ✅ | ✅ `/v1/auth/callback` | ✅ `handlers::auth_callback` | `lcc.members`, `lcc.oauth_tokens` | ✅ |
| `auth.refresh` | POST | `/api/v1/auth/refresh` | ✅ | ✅ `/v1/auth/refresh` | ✅ `handlers::auth_refresh` | `lcc.oauth_tokens` | ✅ |
| `auth.logout` | POST | `/api/v1/auth/logout` | ✅ | ✅ `/v1/auth/logout` | ✅ `handlers::auth_logout` | `lcc.oauth_tokens` | ✅ |
| `members.me.get` | GET | `/api/v1/members/me` | ✅ | ✅ `/v1/profile` (proxy) → upstreams implement `/v1/profile_svc/items` ❌ MISMATCH | ⚠️ identity-svc implements `/v1/identity_svc/items` ❌ MISMATCH | `lcc.members` | ⚠️ scaffold, not members/me |
| `members.me.settings.patch` | PATCH | `/api/v1/members/me/settings` | ✅ | ✅ `/v1/profile` (proxy) | ❌ scaffold doesn't expose `/v1/identity_svc/items/{id}` PATCH | `lcc.members` | ⚠️ |
| `members.{id}.export` | GET | `/api/v1/members/{memberId}/export` | ✅ | ⚠️ not declared in api-gateway router — only `/v1/profile/*path` proxies `/members` traffic to profile-svc which has no export route | ❌ | `lcc.members` + cascade | ❌ MISSING |
| `members.{id}.briefing.today` | GET | `/api/v1/members/{memberId}/briefing/today` | ✅ | ⚠️ proxies via `/v1/profile/*path` | ❌ orchestrator has no `/v1/orchestrator/briefing/today` route | `lcc.briefings` | ❌ MISSING |

## C. PROFILE (Profile Svc) (6 paths, 7 ops)

| Endpoint ID | Method | Canonical Path | FE calls | Gateway | Backend route | DB | Implementation |
|---|---|---|---|---|---|---|---|
| `profile.snapshots.latest` | GET | `/api/v1/members/{memberId}/profile/snapshots/latest` | ✅ | ⚠️ proxies via `/v1/profile/*path` → upstream path is `/v1/profile_svc/items/latest` (does not exist) | ❌ profile-svc scaffold `/v1/profile_svc/items/:id` | `lcc.profile_snapshots` | ⚠️ scaffold only |
| `profile.audit` | POST | `/api/v1/members/{memberId}/profile/audit` | ✅ | ⚠️ | ❌ | `lcc.profile_snapshots`, `lcc.profile_audits` | ⚠️ |
| `profile.edit_drafts` | POST | `/api/v1/members/{memberId}/profile/edit-drafts` | ✅ | ⚠️ | ❌ | (uses `lcc.kb_records` indirectly) | ⚠️ |
| `profile.edit_drafts.{id}.approve` | POST | `/api/v1/members/{memberId}/profile/edit-drafts/{draftId}/approve` | ✅ | ⚠️ | ❌ | (creates Approval row) | ⚠️ |
| `profile.strength_history` | GET | `/api/v1/members/{memberId}/profile/strength-history?range=…` | ✅ | ⚠️ | ❌ | `lcc.profile_snapshots` | ⚠️ |
| `profile.{snapshot_id}.get` | GET | `/api/v1/members/{memberId}/profile/snapshots/{snapshotId}` | (FE doesn't call) | ⚠️ | ❌ | `lcc.profile_snapshots` | ❌ MISSING (design-required but unused by FE) |

## D. CONTENT (Content Svc) (9 paths, 14 ops)

| Endpoint ID | Method | Canonical Path | FE calls | Gateway | Backend route | DB | Implementation |
|---|---|---|---|---|---|---|---|
| `content.list` | GET | `/api/v1/members/{memberId}/content?status=…` | ✅ | ⚠️ | ❌ | `lcc.content_items` | ⚠️ |
| `content.create` | POST | `/api/v1/members/{memberId}/content` | ✅ | ⚠️ | ❌ | `lcc.content_items` | ⚠️ |
| `content.{id}.get` | GET | `/api/v1/members/{memberId}/content/{contentId}` | ✅ | ⚠️ | ❌ | `lcc.content_items` | ⚠️ |
| `content.{id}.update` | PATCH | `/api/v1/members/{memberId}/content/{contentId}` | ✅ | ⚠️ | ❌ | `lcc.content_items` | ⚠️ |
| `content.{id}.delete` | DELETE | `/api/v1/members/{memberId}/content/{contentId}` | ✅ typed | ⚠️ | ❌ | `lcc.content_items` (soft-delete) | ⚠️ |
| `content.compose` | POST | `/api/v1/members/{memberId}/content/compose` | ✅ | ⚠️ | ❌ | (calls `ai-worker.draft_content` gRPC) | ⚠️ |
| `content.{id}.quality_check` | POST | `/api/v1/members/{memberId}/content/{contentId}/quality-check` | ✅ | ⚠️ | ❌ | `lcc.content_items.quality_loop_count` | ⚠️ |
| `content.{id}.submit_for_approval` | POST | `/api/v1/members/{memberId}/content/{contentId}/submit-for-approval` | ✅ | ⚠️ | ❌ | `lcc.approvals` | ⚠️ |
| `content.{id}.schedule` | POST | `/api/v1/members/{memberId}/content/{contentId}/schedule` | ✅ | ⚠️ | ❌ | `lcc.content_items.scheduled_at` | ⚠️ |
| `content.calendar` | GET | `/api/v1/members/{memberId}/content/calendar` | ✅ | ⚠️ | ❌ | `lcc.content_items` (indexed by `scheduled_at WHERE state='scheduled'`) | ⚠️ |
| `content.{id}.approve` | POST | `/api/v1/members/{memberId}/content/{contentId}/approve` | (Design §18.3 row 6) | ⚠️ | ❌ | `lcc.approvals` + permit_token flow | ❌ MISSING (design §18.3 row 6, frontend OpenAPI missing — net gap) |
| `content.{id}.reject` | POST | `/api/v1/members/{memberId}/content/{contentId}/reject` | (Design §18.3 row 7) | ⚠️ | ❌ | `lcc.approvals` | ❌ MISSING (frontend uses bulk on /approvals/{id}/decide) |

## E. ENGAGEMENT (Engagement Svc) (6 paths, 8 ops)

| Endpoint ID | Method | Canonical Path | FE calls | Gateway | Backend route | DB | Implementation |
|---|---|---|---|---|---|---|---|
| `engagement.inbox` | GET | `/api/v1/members/{memberId}/engagement/inbox` | ✅ | ⚠️ | ❌ | `lcc.inbound_messages` | ⚠️ |
| `engagement.queue` | GET | `/api/v1/members/{memberId}/engagement/queue` | ✅ | ⚠️ | ❌ | `lcc.engagement_replies` (renamed from `engagement_task` in design) | ⚠️ |
| `engagement.tasks.create` | POST | `/api/v1/members/{memberId}/engagement/tasks` | (Design §18.4) | ⚠️ | ❌ | `lcc.engagement_replies` | ❌ MISSING (frontend also does not expose) |
| `engagement.tasks.{id}.draft_reply` | POST | `/api/v1/members/{memberId}/engagement/tasks/{taskId}/draft-reply` | ✅ | ⚠️ | ❌ | `lcc.engagement_replies.draft_body` | ⚠️ |
| `engagement.tasks.{id}.approve` | POST | `/api/v1/members/{memberId}/engagement/tasks/{taskId}/approve` | ✅ | ⚠️ | ❌ | `lcc.approvals` | ⚠️ |
| `engagement.tasks.{id}.dismiss` | POST | `/api/v1/members/{memberId}/engagement/tasks/{taskId}/dismiss` | ✅ typed | ⚠️ | ❌ | `lcc.engagement_replies.status='dismissed'` | ⚠️ |

## F. NETWORK / CRM (Network/CRM Svc) (7 paths, 10 ops)

| Endpoint ID | Method | Canonical Path | FE calls | Gateway | Backend route | DB | Implementation |
|---|---|---|---|---|---|---|---|
| `contacts.list` | GET | `/api/v1/members/{memberId}/contacts` | ✅ | ⚠️ | ❌ | `lcc.contacts` | ⚠️ |
| `contacts.create` | POST | `/api/v1/members/{memberId}/contacts` | ✅ | ⚠️ | ❌ | `lcc.contacts` | ⚠️ |
| `contacts.{id}.get` | GET | `/api/v1/members/{memberId}/contacts/{contactId}` | ✅ | ⚠️ | ❌ | `lcc.contacts` | ⚠️ |
| `contacts.{id}.update` | PATCH | `/api/v1/members/{memberId}/contacts/{contactId}` | ✅ | ⚠️ | ❌ | `lcc.contacts` | ⚠️ |
| `contacts.{id}.interactions.list` | GET | `/api/v1/members/{memberId}/contacts/{contactId}/interactions` | ✅ | ⚠️ | ❌ | (interactions table missing; contacts.interactions jsonb) | ⚠️ |
| `contacts.{id}.interactions.create` | POST | `/api/v1/members/{memberId}/contacts/{contactId}/interactions` | ✅ | ⚠️ | ❌ | (above) | ⚠️ |
| `contacts.stale` | GET | `/api/v1/members/{memberId}/contacts/stale` | ✅ | ⚠️ | ❌ | `lcc.contacts` (uses `last_interaction_at` + tier-derived threshold — migration 0016 fixed) | ⚠️ |
| `contacts.{id}.purge` | POST | `/api/v1/members/{memberId}/contacts/{contactId}/purge` | ✅ typed | ⚠️ | ❌ | `lcc.contacts` (hard delete + cascade) | ⚠️ |

## G. OPPORTUNITY (Opportunity Svc) (8 paths, 11 ops)

| Endpoint ID | Method | Canonical Path | FE calls | Gateway | Backend route | DB | Implementation |
|---|---|---|---|---|---|---|---|
| `opportunities.list` | GET | `/api/v1/members/{memberId}/opportunities?status=…&type=…&min_fit_score=…` | ✅ | ⚠️ | ❌ | `lcc.opportunities` | ⚠️ |
| `opportunities.discover` | POST | `/api/v1/members/{memberId}/opportunities/discover` | ✅ | ⚠️ | ❌ | (triggers `opportunity-intel.discover` gRPC, writes `lcc.opportunities`) | ⚠️ |
| `opportunities.{id}.get` | GET | `/api/v1/members/{memberId}/opportunities/{opportunityId}` | ✅ | ⚠️ | ❌ | `lcc.opportunities` | ⚠️ |
| `opportunities.{id}.qualify` | POST | `/api/v1/members/{memberId}/opportunities/{opportunityId}/qualify` | (Design §18.6) | ⚠️ | ❌ | `lcc.opportunities` | ⚠️ |
| `opportunities.{id}.draft_application` | POST | `/api/v1/members/{memberId}/opportunities/{opportunityId}/draft-application` | ✅ | ⚠️ | ❌ | (returns `ApplicationDraft`; doesn't write) | ⚠️ |
| `opportunities.{id}.applications.submit` | POST | `/api/v1/members/{memberId}/opportunities/{opportunityId}/applications` | (Design §18.6 row 5) | ⚠️ | ❌ | `lcc.applications` (Tier-5 — gated) | ⚠️ |
| `opportunities.{id}.draft_proposal` | POST | `/api/v1/members/{memberId}/opportunities/{opportunityId}/draft-proposal` | ✅ | ⚠️ | ❌ | `lcc.applications` | ⚠️ |
| `opportunities.{id}.send_proposal` | POST | `/api/v1/members/{memberId}/opportunities/{opportunityId}/send-proposal` | (Design §18.6 row 7) | ⚠️ | ❌ | `lcc.applications` (Tier-5) | ⚠️ |
| `opportunities.{id}.advance` | POST | `/api/v1/members/{memberId}/opportunities/{opportunityId}/advance` | (Design §18.6 row 8) | ⚠️ | ❌ | `lcc.opportunities.status` | ⚠️ |

## H. SEQUENCES / OUTREACH (Outreach Svc) (9 paths, 12 ops)

| Endpoint ID | Method | Canonical Path | FE calls | Gateway | Backend route | DB | Implementation |
|---|---|---|---|---|---|---|---|
| `sequences.list` | GET | `/api/v1/members/{memberId}/sequences` | ✅ | ⚠️ | ❌ | `lcc.sequences` | ⚠️ |
| `sequences.create` | POST | `/api/v1/members/{memberId}/sequences` | ✅ | ⚠️ | ❌ | `lcc.sequences`, `lcc.sequence_steps` | ⚠️ |
| `sequences.{id}.get` | GET | `/api/v1/members/{memberId}/sequences/{sequenceId}` | ✅ | ⚠️ | ❌ | `lcc.sequences` + `lcc.sequence_steps` | ⚠️ |
| `sequences.{id}.approve` | POST | `/api/v1/members/{memberId}/sequences/{sequenceId}/approve` | ✅ | ⚠️ | ❌ | `lcc.sequences.status`, `lcc.approvals` | ⚠️ |
| `sequences.{id}.pause` | POST | `/api/v1/members/{memberId}/sequences/{sequenceId}/pause` | ✅ | ⚠️ | ❌ | `lcc.sequences.status='paused'` | ⚠️ |
| `sequences.{id}.resume` | POST | `/api/v1/members/{memberId}/sequences/{sequenceId}/resume` | ✅ typed | ⚠️ | ❌ | `lcc.sequences.status='active'` | ⚠️ |
| `sequences.{id}.abandon` | POST | `/api/v1/members/{memberId}/sequences/{sequenceId}/abandon` | ✅ typed | ⚠️ | ❌ | `lcc.sequences.status='abandoned'` | ⚠️ |
| `sequences.{id}.steps.{id}.approve` | POST | `/api/v1/members/{memberId}/sequences/{sequenceId}/steps/{stepId}/approve` | ✅ | ⚠️ | ❌ | `lcc.sequence_steps.approval_id`, `lcc.approvals` | ⚠️ |
| `sequences.templates.list` | GET | `/api/v1/members/{memberId}/sequences/templates?persona=…` | ✅ | ⚠️ | ❌ | `lcc.message_templates` | ⚠️ |
| `sequences.templates.create` | POST | `/api/v1/members/{memberId}/sequences/templates` | ✅ typed | ⚠️ | ❌ | `lcc.message_templates` | ⚠️ |

## I. KB (KB Svc) (4 paths, 5 ops)

| Endpoint ID | Method | Canonical Path | FE calls | Gateway | Backend route | DB | Implementation |
|---|---|---|---|---|---|---|---|
| `kb.records.list` | GET | `/api/v1/members/{memberId}/kb/records?category=…` | ✅ | ⚠️ | ❌ | `lcc.kb_records` | ⚠️ |
| `kb.records.create` | POST | `/api/v1/members/{memberId}/kb/records` | ✅ | ⚠️ | ❌ | `lcc.kb_records` (triggers re-embed via `kb.record.created` event) | ⚠️ |
| `kb.records.{id}.get` | GET | `/api/v1/members/{memberId}/kb/records/{recordId}` | ✅ | ⚠️ | ❌ | `lcc.kb_records` | ⚠️ |
| `kb.records.{id}.update` | PATCH | `/api/v1/members/{memberId}/kb/records/{recordId}` | ✅ | ⚠️ | ❌ | `lcc.kb_records` | ⚠️ |
| `kb.records.{id}.delete` | DELETE | `/api/v1/members/{memberId}/kb/records/{recordId}` | ✅ typed | ⚠️ | ❌ | `lcc.kb_records` (soft-delete, removes embedding) | ⚠️ |

## J. ANALYTICS (Analytics Svc) (9 paths, 9 ops)

| Endpoint ID | Method | Canonical Path | FE calls | Gateway | Backend route | DB | Implementation |
|---|---|---|---|---|---|---|---|
| `analytics.content` | GET | `/api/v1/members/{memberId}/analytics/content` | ✅ | ⚠️ | ❌ | `lcc.post_metrics`, OLAP `fact_content_metrics` | ⚠️ |
| `analytics.profile` | GET | `/api/v1/members/{memberId}/analytics/profile` | ✅ | ⚠️ | ❌ | `lcc.profile_snapshots`, OLAP `fact_profile_event` | ⚠️ |
| `analytics.network` | GET | `/api/v1/members/{memberId}/analytics/network` | ✅ | ⚠️ | ❌ | `lcc.contacts`, OLAP `fact_network_event` | ⚠️ |
| `analytics.outreach` | GET | `/api/v1/members/{memberId}/analytics/outreach` | ✅ | ⚠️ | ❌ | `lcc.sequence_steps`, OLAP `fact_outreach_send` | ⚠️ |
| `analytics.funnel.job` | GET | `/api/v1/members/{memberId}/analytics/funnel/job` | ✅ | ⚠️ | ❌ | `lcc.opportunities`, OLAP `fact_opportunity_funnel` | ⚠️ |
| `analytics.funnel.client` | GET | `/api/v1/members/{memberId}/analytics/funnel/client` | ✅ | ⚠️ | ❌ | (same) | ⚠️ |
| `analytics.account_health` | GET | `/api/v1/members/{memberId}/analytics/account-health` | ✅ | ⚠️ | ❌ | `lcc.account_health_snapshots` (calls `scoring-intel.compute_h_c`) | ⚠️ |
| `analytics.digest.weekly` | GET | `/api/v1/members/{memberId}/analytics/digest/weekly` | ✅ | ⚠️ | ❌ | (aggregates) | ⚠️ |
| `analytics.digest.monthly` | GET | `/api/v1/members/{memberId}/analytics/digest/monthly` | ✅ | ⚠️ | ❌ | (aggregates) | ⚠️ |

## K. APPROVALS (Approval Svc) (3 paths, 4 ops)

| Endpoint ID | Method | Canonical Path | FE calls | Gateway | Backend route | DB | Implementation |
|---|---|---|---|---|---|---|---|
| `approvals.list` | GET | `/api/v1/members/{memberId}/approvals` | ✅ | ⚠️ | ❌ | `lcc.approvals` | ⚠️ |
| `approvals.{id}.get` | GET | `/api/v1/members/{memberId}/approvals/{approvalId}` | ✅ | ⚠️ | ❌ | `lcc.approvals` | ⚠️ |
| `approvals.{id}.decide` | POST | `/api/v1/members/{memberId}/approvals/{approvalId}/decide` | ✅ | ⚠️ | ❌ | `lcc.approvals.status` (calls Compliance Governor inline) | ⚠️ |
| `approvals.bulk_decide` | POST | `/api/v1/members/{memberId}/approvals/bulk-decide` | (Design §18.8) | ⚠️ | ❌ | `lcc.approvals` (single resource_type only) | ⚠️ |

## L. AUDIT (Audit Svc) (1 path, 1 op)

| Endpoint ID | Method | Canonical Path | FE calls | Gateway | Backend route | DB | Implementation |
|---|---|---|---|---|---|---|---|
| `audit.list` | GET | `/api/v1/members/{memberId}/audit` | (Admin only) | ⚠️ | ❌ | `lcc_audit.events` (Auditor/Admin role required) | ⚠️ |

## M. ADMIN / COMPLIANCE (4 paths, 7 ops)

| Endpoint ID | Method | Canonical Path | FE calls | Gateway | Backend route | DB | Implementation |
|---|---|---|---|---|---|---|---|
| `admin.compliance.config_versions.list` | GET | `/api/v1/admin/compliance/config-versions` | ✅ | ⚠️ proxies via `/v1/admin/governor/evaluate` (mismatched path) — no `/v1/admin/compliance/config-versions` route | ❌ | `lcc.compliance_config_versions` | ❌ MISSING |
| `admin.compliance.config_versions.create` | POST | `/api/v1/admin/compliance/config-versions` | ✅ | ⚠️ | ❌ | `lcc.compliance_config_versions` (status=`draft`) | ❌ MISSING |
| `admin.compliance.config_versions.{id}.activate` | POST | `/api/v1/admin/compliance/config-versions/{versionId}/activate` | ✅ | ⚠️ | ❌ | `lcc.compliance_config_versions.status='active'` + `two_reviewer_signed_by` | ⚠️ (per audit F-AUDIT-22: 2-reviewer signature check is a stub) |
| `admin.compliance.restrictions.{member}.get` | GET | `/api/v1/admin/compliance/restrictions/{memberId}` | ✅ | ⚠️ | ❌ | `lcc.restrictions` | ⚠️ |
| `admin.compliance.restrictions.{member}.clear` | POST | `/api/v1/admin/compliance/restrictions/{memberId}/clear` | ✅ | ⚠️ | ❌ | `lcc.restrictions.cleared_at` | ⚠️ |

---

## Coverage Matrix (per-domain roll-up)

| Domain | Design §18 ops | Frontend OpenAPI ops | Backend OpenAPI ops | Backend routes (real) | Canonical ops |
|---|---:|---:|---:|---:|---:|
| Infra | n/a | 0 | 2 | 2 | 3 |
| Auth | 4 | 4 | 0 | 4 (handlers, /v1/auth/*) | 4 |
| Members | 2 | 2 | 0 | 0 (scaffold) | 3 |
| Profile | 6 | 5 | 0 | 0 (scaffold) | 7 |
| Content | 11 | 10 | 1 | 1 (scaffold) | 14 |
| Engagement | 6 | 6 | 0 | 0 (scaffold) | 8 |
| Contacts | 7 | 7 | 0 | 0 (scaffold) | 10 |
| Opportunity | 8 | 5 | 0 | 0 (scaffold) | 11 |
| Sequences | 9 | 7 | 0 | 0 (scaffold) | 12 |
| KB | 4 | 4 | 0 | 0 (scaffold) | 5 |
| Analytics | 9 | 9 | 0 | 0 (scaffold) | 9 |
| Approvals | 3 | 3 | 1 (template only) | 0 (scaffold) | 4 |
| Audit | 0 | 0 | 0 | 0 | 1 (canonical-only — admin/auditor visibility) |
| Admin | 5 | 5 | 1 (different path) | 0 | 7 |
| **TOTAL** | **~74** | **67** | **5** | **7** | **98** |

(Note: "scaffold" = byte-identical generic `Entity`/`EntityCreate` router with no domain logic — see audit §3.0.)

## Cross-Cutting Issues

### Namespace collisions in current gateway routing

```
Gateway router (api-gateway/src/http/router.rs):
  /v1/profile/*path    → proxy_request
  /v1/content/*path    → proxy_request
  /v1/engagement/*path → proxy_request
  /v1/network/*path    → proxy_request    ← inconsistent with FE (FE uses /contacts, not /network)
  /v1/opportunity/*path→ proxy_request    ← singular, vs canonical plural "opportunities"
  /v1/outreach/*path   → proxy_request    ← inconsistent with FE (FE uses /sequences, not /outreach)
  /v1/analytics/*path  → proxy_request
  /v1/approvals/*path  → proxy_request
  /v1/audit/*path      → proxy_request
  /v1/admin/governor/evaluate  ← only one admin route, doesn't match /admin/compliance/* family
```

```
Backend service routers (all 12 services):
  /v1/<service>_svc          → handlers::root (placeholder JSON)
  /v1/<service>_svc/items    → generic CRUD (scaffold)
  /v1/<service>_svc/items/:id → generic CRUD (scaffold)
```

The gateway proxy path **does not match** the upstream route. Forwarding verbatim causes every backend route to return a 404 from the gateway (`proxy/mod.rs` honesty note calls this out explicitly). The current "fix" is to surface honest 404s rather than fabricated 200s.

### Frontend path-prefix inconsistency

```
Frontend apiFetch() calls hit these path patterns:
  /auth/...                  (auth; correct)
  /members/...               (correct)
  /admin/...                 (admin; correct)
  /approvals                 ← DOES NOT INCLUDE /members/{id}/  ❌
  /opportunities             ← DOES NOT INCLUDE /members/{id}/  ❌
  /content                   ← DOES NOT INCLUDE /members/{id}/  ❌
  /profile/audit             ← DOES NOT INCLUDE /members/{id}/  ❌
  /members/{id}/profile/snapshots/latest   (correct)
```

(Confirmed in `apps/web-dashboard/src/lib/api/approval.ts`, `apps/web-dashboard/src/lib/api/opportunity.ts`, `apps/web-dashboard/src/lib/api/profile.ts` — uses both forms. The OpenAPI spec is consistent; the implementation has gaps. The frontend apiFetch `/approvals/...` calls hit the Next origin's 404 because there's no Next rewrite, while `/members/{id}/approvals/{id}/decide` is what would work.)

---

## Detailed Gap Inventory

### G1. Three competing OpenAPI documents

| Document | Lines | Paths | Operations | Namespace |
|---|---:|---:|---:|---|
| `LCC_Backend/schemas/openapi/openapi.yaml` | 190 | 6 | 5 | `/v1/...` |
| `LCC_Frontend/lcc/schemas/openapi/api-gateway.yaml` | 1,919 | 55 | 67 | no prefix |
| `contract_audit/openapi/lcc-api-canonical.yaml` (this audit) | 933 | 70 | 82 | `/api/v1/...` |

**Resolution:** canonical (`lcc-api-canonical.yaml`) supersedes both. The two existing documents should be deprecated by a CI policy (no OpenAPI may exist outside `schemas/openapi/lcc-api-canonical.yaml`).

### G2. Gateway path rewriting gap

The api-gateway's proxy forwards `/v1/profile/...` verbatim to `profile-svc`. The backend service listens at `/v1/profile_svc/...`. There is **no path rewrite** in the gateway and the audit report explicitly states none should be added (it would mask the gap with another layer of fiction).

**Resolution options** (any ONE of these):
1. **Update the backend routers** so every service registers `/v1/<domain>/...` (canonical) — requires editing 11 services to delete the generic CRUD scaffold and replace with domain routes.
2. **Add a gateway path-rewrite** mapping `/v1/profile/...` → `/v1/profile_svc/...` — explicitly rejected by the audit.
3. **Move all real backend routes** under `/v1/<service>_svc/<domain>/...` and have the gateway proxy forward `/v1/<domain>/...` → `/v1/<service>_svc/<domain>/...` — requires a rewrite but centralizes it.

Recommended: **option 1** (canonical namespace = single source of truth everywhere; rewrite only the internal pre-canonical scaffold path).

### G3. Missing endpoints (design-required, FE never calls)

| Design § | Endpoint | Why missing |
|---|---|---|
| §18.3 | `POST /content/{id}/approve` | The FE uses the generic `/members/{id}/approvals/{id}/decide` flow, but the design explicitly defines a content-specific convenience endpoint. Either remove from design or implement it as an alias. |
| §18.3 | `POST /content/{id}/reject` | Same — design has it, FE uses generic approvals. |
| §18.4 | `POST /engagement/tasks` | Design says daily-ritual queue generation creates tasks; FE only has a `GET /engagement/queue` reader. |
| §18.6 | `POST /opportunities/{id}/qualify` | Design says "compute φ and move to qualified"; FE doesn't expose this. |
| §18.6 | `POST /opportunities/{id}/applications` | Tier-5 gated submit; FE only has `draft-application`. |
| §18.6 | `POST /opportunities/{id}/send-proposal` | Tier-5 gated proposal send; FE only has `draft-proposal`. |
| §18.6 | `POST /opportunities/{id}/advance` | Manual pipeline advance; not exposed in FE. |
| §18.8 | `POST /approvals/bulk-decide` | Design says bulk within one resource_type; FE doesn't have this. |

### G4. Frontend endpoint calls that have no backend implementation

**All 49** of the FE's `apiFetch()` / query-mutation calls land on backend routes that are currently `byte-identical 692-LOC scaffolds`. Specifically:

- `apps/web-dashboard/src/lib/api/admin.ts` → 5 calls → all hit identity-svc / audit-svc / compliance-governor with no real handlers
- `apps/web-dashboard/src/lib/api/approval.ts` → 3 calls → no Approval Svc handlers
- `apps/web-dashboard/src/lib/api/content.ts` → 9 calls → no Content Svc handlers
- `apps/web-dashboard/src/lib/api/engagement.ts` → 5 calls → no Engagement Svc handlers
- `apps/web-dashboard/src/lib/api/kb.ts` → 5 calls → no KB handlers
- `apps/web-dashboard/src/lib/api/members.ts` → 6 calls → only `/members/me` would map to Identity Svc (and only via proxy)
- `apps/web-dashboard/src/lib/api/network.ts` → 6 calls → no Network/CRM Svc handlers
- `apps/web-dashboard/src/lib/api/opportunity.ts` → 5 calls → no Opportunity Svc handlers
- `apps/web-dashboard/src/lib/api/outreach.ts` → 5 calls → no Outreach Svc handlers
- `apps/web-dashboard/src/lib/api/profile.ts` → 7 calls → no Profile Svc handlers
- `apps/web-dashboard/src/lib/api/analytics.ts` → 10 calls → no Analytics Svc handlers
- `apps/web-dashboard/src/lib/api/auth.ts` → 2 calls → only OAuth start/callback are real

This is consistent with the audit report's headline finding: **0 / 60 design endpoints are implemented**.

### G5. Backend routes that exist but have no legitimate consumer

Every backend service router currently exposes:
```
GET    /v1/<svc>_svc                  → handlers::root ({"status":"ok"})
GET    /v1/<svc>_svc/items            → scaffold CRUD list
POST   /v1/<svc>_svc/items            → scaffold CRUD create
GET    /v1/<svc>_svc/items/:id        → scaffold CRUD get
PATCH  /v1/<svc>_svc/items/:id        → scaffold CRUD update
DELETE /v1/<svc>_svc/items/:id        → scaffold CRUD delete
```

All 60 of these routes are **dead code** — they are byte-identical scaffolds that return generic `Entity` objects with no domain semantics. There is no consumer (FE never calls `/v1/<svc>_svc/...`). They must be deleted once real domain routes are added under `/v1/<domain>/...`.

### G6. Service ownership mapping (after canonicalization)

| Canonical domain | Owning service | Runtime | Auth scope |
|---|---|---|---|
| `/api/v1/auth/*` | identity-svc | Rust | public (start/callback), JWT (refresh/logout) |
| `/api/v1/members/*` | identity-svc | Rust | JWT, self-only |
| `/api/v1/profile/*` | profile-svc | Rust (CRUD) + ai-worker gRPC (audit/edit drafts) | JWT, self-only |
| `/api/v1/content/*` | content-svc | Rust (CRUD + state machine) + ai-worker gRPC (compose) + voice-intel (embed) | JWT, self-only |
| `/api/v1/engagement/*` | engagement-svc | Rust + ai-worker gRPC (draft reply) + scoring-intel (ρ) | JWT, self-only |
| `/api/v1/contacts/*` | network-crm-svc | Rust | JWT, self-only |
| `/api/v1/opportunities/*` | opportunity-svc | Rust (CRUD) + opportunity-intel gRPC (φ, discover) | JWT, self-only |
| `/api/v1/sequences/*` | outreach-svc | Rust (state machine) + ai-worker gRPC (draft outreach) | JWT, self-only |
| `/api/v1/kb/*` | profile-svc (or dedicated kb-svc) | Rust (CRUD) + kb-intel gRPC (embed) | JWT, self-only |
| `/api/v1/analytics/*` | analytics-svc | Rust (read-only role) | JWT, self-only (admin: any) |
| `/api/v1/briefing/*` | orchestrator | Rust | JWT, self-only |
| `/api/v1/approvals/*` | approval-svc | Rust + compliance-governor gRPC (inline evaluate) | JWT, self-only |
| `/api/v1/audit/*` | audit-svc | Rust (read-only role) | JWT, role=Auditor or Admin |
| `/api/v1/admin/compliance/*` | compliance-governor (config) + identity-svc (restrictions) | Rust | JWT, role=Admin |

### G7. Database ↔ API entity mapping

| API entity | DB table | Migration |
|---|---|---|
| `Member` | `lcc.members` | 0003 |
| (tokens) | `lcc.oauth_tokens` | 0003 |
| (consents) | `lcc.consents` | 0003 |
| `ProfileSnapshot` | `lcc.profile_snapshots` | 0004 |
| (audit) | `lcc.profile_audits` | 0004 |
| `KbRecord` | `lcc.kb_records` | 0005 |
| (chunks) | `lcc.kb_record_chunks` | 0005 |
| `ContentItem` | `lcc.content_items` | 0006 |
| (metrics) | `lcc.post_metrics` | 0006 |
| `InboxItem` | `lcc.inbound_messages` | 0007 |
| `EngagementTask` | `lcc.engagement_replies` | 0007 (note: design has `engagement_task`; DB has `engagement_replies` — naming inconsistency to resolve) |
| `Sequence` | `lcc.sequences` | 0007 |
| `SequenceStep` | `lcc.sequence_steps` | 0007 + 0015 (governance cols) |
| `Contact` | `lcc.contacts` | 0008 |
| (lists) | `lcc.network_lists`, `lcc.network_list_memberships`, `lcc.tags` | 0008 |
| `Opportunity` | `lcc.opportunities` | 0009 |
| (signals) | `lcc.opportunity_signals` | 0009 |
| (drafts) | `lcc.outreach_drafts` | 0009 |
| `ComplianceConfigVersion` | `lcc.compliance_config_versions` | 0010 |
| `AccountHealth` | `lcc.account_health_snapshots` | 0010 |
| `DailyQuotaCounter` | `lcc.daily_caps` | 0010 |
| (cooldowns) | `lcc.cooldown_rules` | 0010 |
| `Restriction` | `lcc.restrictions` | 0010 |
| `Approval` | `lcc.approvals` | 0011 |
| `AuditEvent` | `lcc_audit.events` | 0011 |
| `IdempotencyKey` | `lcc.idempotency_keys` | 0011 |
| (session pacing) | `lcc.session_pacing` | 0012 |
| `Briefing` | `lcc.briefings` | 0012 |
| (outbox) | `lcc.outbox` | 0013 |
| (analytics rollup) | `lcc.analytics_member_daily` (materialized view) | 0013 |
| `Application` | `lcc.applications` | 0014 |
| `MessageTemplate` | `lcc.message_templates` | 0014 |
| (sequence step governance) | `lcc.sequence_steps` (added cols) | 0015 |
| (staleness) | `lcc.contacts` (added cols) | 0016 |

**Missing / partial tables required by the canonical contract:**

| Design § | Required table / column | Status |
|---|---|---|
| §11 | `lcc.sequence_steps.step_status` enum extension | ⚠️ partially — migration 0015 adds columns but uses TEXT not ENUM |
| §11.6 | `lcc.contacts.third_party_ttl_at` | ❌ missing |
| §11.7 | `lcc.companies` table (linked from contacts) | ❌ missing — design §11.6 declares it but no migration creates it |
| §11.10 | `lcc.applications.documents`, `messages` JSONB columns | ⚠️ partial — payload JSONB exists, no separate documents/messages columns |
| §11.12 | `lcc.engagement_replies.draft_body` | ✅ present |
| §22 | `lcc.opportunity_signals.observed_at` | ✅ present |
| §26 | `lcc.outbox` for transactional outbox | ✅ present |

**Missing canonical tables:**
- `lcc.companies` — design §11.6 declares it; no migration creates it; `lcc.contacts.company_id` references a non-existent table.
- `lcc.interactions` — design §11.5 says "Conversation History (log of interactions)"; current migration 0008 doesn't create an interactions table; interactions are stored as embedded array on contact or only on `inbound_messages`.
- `lcc.compliance_audit_chain` — design §17 calls for an integrity-check hash chain, but it's only implemented in `lcc_audit.events.checksum_sha256`; no separate chain-validation table.

### G8. Realtime channels

The design §19 specifies **5 channels**: `/ws/briefing`, `/ws/approvals`, `/ws/engagement`, `/ws/compliance`, `/ws/sequence`. The frontend `packages/realtime/src/channels/` declares **6 channels** (the FE adds `/ws/integration`). The backend has **zero** — the only WebSocket server is the integration-gateway's Track B (`browser-extension`, on port 8443, ingress at `browser.lcc.example/ws`), which is the Browser-Extension human-assist channel, not the dashboard's realtime channel.

| Channel | Design | Frontend | Backend | Status |
|---|---|---|---|---|
| `/ws/briefing` | ✅ | ✅ `subscribeBriefingChannel` | ❌ MISSING | 0 callers on backend |
| `/ws/approvals` | ✅ | ✅ `subscribeApprovalsChannel` | ❌ MISSING | 0 callers on backend |
| `/ws/engagement` | ✅ | ✅ `subscribeEngagementChannel` | ❌ MISSING | 0 callers on backend |
| `/ws/compliance` | ✅ | ✅ `subscribeComplianceChannel` | ❌ MISSING | 0 callers on backend |
| `/ws/sequence` | ✅ | ✅ `subscribeSequenceChannel` | ❌ MISSING | 0 callers on backend |
| `/ws/integration` | ❌ (extension only) | ✅ | ✅ (Track B; for browser extension, not dashboard) | OK by design intent |

See `realtime/lcc-realtime-contract.yaml` for the canonical realtime schema (event names, payloads, auth, reconnection).

### G9. Gateway port allocations (confirmed)

| Service | Port | Evidence |
|---|---:|---|
| api-gateway | 8080 | `engine/core/services/api-gateway/src/config.rs`, `infra/k8s/base/api-gateway.yaml` |
| orchestrator | 8081 | `infra/k8s/base/orchestrator.yaml` |
| profile-svc | 8082 | `infra/k8s/base/profile-svc.yaml` |
| content-svc | 8083 | `infra/k8s/base/content-svc.yaml` |
| engagement-svc | 8084 | `infra/k8s/base/engagement-svc.yaml` |
| network-crm-svc | 8085 | `infra/k8s/base/network-crm-svc.yaml` |
| opportunity-svc | 8086 | `infra/k8s/base/opportunity-svc.yaml` |
| outreach-svc | 8087 | `infra/k8s/base/outreach-svc.yaml` |
| analytics-svc | 8088 | `infra/k8s/base/analytics-svc.yaml` |
| approval-svc | 8089 | `infra/k8s/base/approval-svc.yaml` |
| identity-svc | 8090 | `infra/k8s/base/identity-svc.yaml` |
| audit-svc | 8091 | `infra/k8s/base/audit-svc.yaml` |
| compliance-governor | 8080 (HTTP) / 50051 (gRPC) | `infra/k8s/base/compliance-governor.yaml` |
| integration-gateway | 8080 (HTTP) / 50051 (gRPC) / 8443 (Track B WS) | `infra/k8s/base/integration-gateway.yaml`, `infra/k8s/ingress/track-b-wss.yaml` |

**Note:** compliance-governor and integration-gateway share port 8080 in their default config. They are differentiated in deployment via NetworkPolicy and by HTTP_PORT env var passed in their k8s manifests, but the *defaults* collide — any local `cargo run` without explicit env vars hits a bind error. This is a known issue flagged by the audit but not remediated.
