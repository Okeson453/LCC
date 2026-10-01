# Redis Schema — OKESON-LCC

| Key pattern                          | Type        | TTL          | Purpose                                      |
|--------------------------------------|-------------|--------------|----------------------------------------------|
| `lcc:idem:{member_id}:{key}`         | STRING      | 7 days       | Idempotency key cache (Integration Layer)    |
| `lcc:idem:meta:{idem_id}`            | HASH        | 7 days       | Idempotency metadata (action_type, etc.)     |
| `lcc:cb:{provider}:{endpoint}`       | STRING      | 5 min        | Circuit breaker state (CLOSED/OPEN/HALF_OPEN) |
| `lcc:cb:failures:{provider}:{endpoint}` | INT     | 5 min        | Failure counter for circuit breaker          |
| `lcc:rate:{member_id}:{window}`      | INT         | window-size  | Per-window rate counter                      |
| `lcc:lock:{member_id}:{action_id}`   | STRING      | 30 sec       | Action mutex lock (prevents race)            |
| `lcc:dedup:{group}:{idem_key}`       | STRING      | 7 days       | Consumer dedup                               |
| `lcc:stream:{topic}`                 | STREAM      | forever      | Event stream (XADD)                          |
| `lcc:dedup:{consumer_group}:{idem}`  | STRING      | 7 days       | Consumer-group-scoped dedup                  |
| `lcc:seq:{member_id}`                | INT         | 1 day        | Session-pacing rolling counter               |
| `lcc:cap:{member_id}:{date}:{action}`| INT         | until date+1 | Daily-cap counter                            |

## Stream topology

```
lcc:stream:member.created
lcc:stream:member.deactivated
lcc:stream:oauth.token_refresh_failed
lcc:stream:profile.snapshot.created
lcc:stream:profile.audit.completed
lcc:stream:kb.record.created
lcc:stream:kb.record.updated
lcc:stream:kb.record.deleted
lcc:stream:voice.sample.added
lcc:stream:content.item.state_changed
lcc:stream:content.item.approved
lcc:stream:engagement.inbound.received
lcc:stream:engagement.reply_drafted
lcc:stream:sequence.reply_detected
lcc:stream:sequence.step.due
lcc:stream:sequence.state_changed
lcc:stream:sequence.step.sent
lcc:stream:opportunity.discovered
lcc:stream:opportunity.qualified
lcc:stream:opportunity.funnel_changed
lcc:stream:approval.decided
lcc:stream:approval.expired
lcc:stream:compliance.config_activated
lcc:stream:compliance.restriction_detected
lcc:stream:compliance.restriction_cleared
lcc:stream:audit.event
```

Consumer groups are service-scoped:
- `ai-worker`, `kb-intel`, `voice-intel`, `enrichment-worker`, `re-embed-worker`, `voice-train-worker`, `opportunity-discovery-worker`, `sequence-step-scheduler`, `staleness-scanner`, `data-purge-worker`, `audit-integrity-worker`.
