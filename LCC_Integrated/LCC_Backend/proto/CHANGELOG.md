# Proto Changelog

This file logs breaking/semver-relevant changes to the proto contract.
Per-package versioning: `lcc.v1.<service>` packages isolate breaking changes.

## 2026-04-22 — v1.0 (initial)

- `lcc.v1.common`: `trace.proto` (TraceContext), `pagination.proto` (PageRequest/Response)
- `lcc.v1.compliance`: `governor.proto` (EvaluateAction, ActivateConfigVersion, PermitMetadata), `admin.proto` (proposals)
- `lcc.v1.integration`: `execute.proto` (Track A/B router, RestrictionSignal)
- `lcc.v1.intelligence`: `ai.proto` (5 drafts + Embed), `opportunity.proto` (ScoreFit, Discover, EnrichCompany), `kb.proto` (EmbedKBRecord, SearchKB), `voice.proto` (sample CRUD), `scoring.proto` (ComputeH_c, ComputePhi, PredictReply)
- `lcc.v1.profile`: `profile.proto` (Snapshot, Audit, EditDraft)
- `lcc.v1.content`: `content.proto` (ContentItem lifecycle)
- `lcc.v1.engagement`: `engagement.proto` (EngagementTask, DailyRitual, Inbox)
- `lcc.v1.network`: `network.proto` (Contact, Company, InteractionLog, GDPR purge)
- `lcc.v1.outreach`: `outreach.proto` (Sequence, SequenceStep, MessageTemplate)
- `lcc.v1.analytics`: `analytics.proto` (metrics, funnels, account-health)
- `lcc.v1.approval`: `approval.proto` (queue, decide, expire)
- `lcc.v1.identity`: `identity.proto` (OAuth2/PKCE, GDPR export/delete)
- `lcc.v1.events`: `events.proto` (envelope + payload schemas)
