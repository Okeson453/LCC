# lcc-api-gateway

The single public-facing HTTP entrypoint for OKESON-LCC. Terminates TLS,
performs JWT validation, enforces rate limits, and forwards requests to
upstream domain services.

## Routes

- `/healthz`, `/readyz`
- `/v1/admin/governor/evaluate` → compliance-governor
- `/v1/profile/*` → profile-svc
- `/v1/content/*` → content-svc
- `/v1/engagement/*` → engagement-svc
- `/v1/network/*` → network-crm-svc
- `/v1/opportunity/*` → opportunity-svc
- `/v1/outreach/*` → outreach-svc
- `/v1/analytics/*` → analytics-svc
- `/v1/approvals/*` → approval-svc
- `/v1/audit/*` → audit-svc
