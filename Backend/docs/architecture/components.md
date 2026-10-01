# Component Catalog

## Core Engine Services

| Service                | Port | Responsibility                                    |
|------------------------|------|---------------------------------------------------|
| api-gateway            | 8080 | Public HTTP entrypoint                            |
| compliance-governor    | 8080 | 8 guards, permit-token issuance                   |
| integration-gateway    | 8080 / 8081 | Verify permit, dispatch Track A/B          |
| orchestrator           | 8081 | Case-state-machine workflows                      |
| profile-svc            | 8082 | Profile CRUD, audits, snapshots                   |
| content-svc            | 8083 | Content items (drafts → scheduled → published)     |
| engagement-svc         | 8084 | Inbound messages, replies, sequences              |
| network-crm-svc        | 8085 | Contacts, lists, tags                             |
| opportunity-svc        | 8086 | Opportunity records, signals, funnel              |
| outreach-svc           | 8087 | Connection requests, DMs, sequence steps          |
| analytics-svc          | 8088 | Metrics, dashboards                               |
| approval-svc           | 8089 | Approval workflow                                 |
| identity-svc           | 8090 | OAuth tokens, member identity                     |
| audit-svc              | 8091 | Audit log + integrity verification               |

## Intelligence Engine Services

| Service                | Port | Responsibility                                    |
|------------------------|------|---------------------------------------------------|
| ai-worker              | 8080 | LLM drafts, RAG, brand guard                      |
| opportunity-intel      | 8090 | φ scoring + discovery                             |
| kb-intel               | 8091 | KB CRUD, chunking, dedup                          |
| voice-intel            | 8092 | Voice fingerprints + sample ingest                |
| scoring-intel          | 8093 | ρ, ab_d, h_c                                      |

## Background Workers

| Worker                          | Job                                     |
|--------------------------------|------------------------------------------|
| briefing-worker                | Generate daily briefings                 |
| sequence-step-scheduler        | Schedule + send sequence steps           |
| opportunity-discovery-worker   | Discover opportunities from cached data  |
| staleness-scanner              | Detect stale KB records + contacts       |
| data-purge-worker              | Purge TTL-expired third-party data       |
| audit-integrity-worker         | Verify audit_log checksum chain          |
| enrichment-worker              | Third-party enrichment (Python)          |
| re-embed-worker                | Re-embed KB on model change (Python)     |
| voice-train-worker             | Train voice centroid per member (Python) |
