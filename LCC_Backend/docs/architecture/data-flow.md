# Data Flow

## Inbound

```
LinkedIn REST APIs ──▶ integration-gateway Track A (REST client)
Browser Extension  ──▶ integration-gateway Track B (WSS oneshot)
                       ▼
        profile_snapshots, inbound_messages
                       ▼
                engagement-svc (idempotent ingest)
```

## Outbound

```
domain svc (e.g., outreach-svc)
    │
    ▼
POST /v1/admin/governor/evaluate
    │   { member_id, action_type, target_kind, target_id, context }
    │
    ▼
compliance-governor
    │   guards 1..8
    │
    ▼
permit-token (JWT, ≤60s TTL, audience="integration-gateway")
    │
    ▼
POST /v1/integration/execute
    │   { permit_token, idempotency_key, payload }
    │
    ▼
integration-gateway
    │   verify JWT → idempotency cache → circuit breaker → retry → dispatch
    │
    ▼
LinkedIn API (Track A)  OR  Browser Extension (Track B)
```

## Async

```
domain svc ──▶ event_bus.publish(topic, payload)
                          │
                          ▼
              lcc:stream:<topic>
                          │
            ┌─────────────┼─────────────┐
            ▼             ▼             ▼
       ai-worker   enrichment-worker  re-embed-worker
            ▼             ▼             ▼
       RAG + draft   profile/snapshot  upsert vectors
            ▼
       audit.event (in tx, by every emitter)
```
