# API Reference

See `schemas/openapi/openapi.yaml` for the canonical OpenAPI spec.

## Endpoints

### Admin
- `POST /v1/admin/governor/evaluate` — pre-flight compliance check.

### Profile
- `GET  /v1/profile`
- `POST /v1/profile/snapshot`

### Content
- `GET  /v1/content/items?state=...`
- `POST /v1/content/items`
- `PATCH /v1/content/items/{id}`

### Engagement
- `GET  /v1/engagement/inbound`
- `POST /v1/engagement/replies`

### Network
- `GET  /v1/network/contacts`
- `POST /v1/network/lists`

### Opportunity
- `GET  /v1/opportunity`
- `POST /v1/opportunity/discover`

### Outreach
- `POST /v1/outreach/connection-request`
- `POST /v1/outreach/dm`

### Analytics
- `GET  /v1/analytics/summary`

### Approvals
- `GET  /v1/approvals/pending`
- `POST /v1/approvals/{id}/decide`

### Audit
- `GET  /v1/audit/events?member_id=...&from=...`
