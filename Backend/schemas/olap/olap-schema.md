# OLAP Schema — OKESON-LCC

Analytics queries run against the same PostgreSQL primary via materialized
views. For Phase 3+, an OLAP warehouse (ClickHouse or Snowflake) is added.

## Materialized Views

### `lcc.analytics_member_daily`
Aggregates per-member daily activity.

```sql
SELECT
  member_id, day, total_actions
FROM lcc.analytics_member_daily;
```

Refresh: every 6 hours via cron, or on-demand by analytics-svc.

### `lcc.analytics_post_performance`
Per-post engagement metrics aggregated.

### `lcc.analytics_sequence_funnel`
Sequence → contact funnel conversion rates.

### `lcc.analytics_compliance_violations`
Aggregate guard deny counts per day.

## Export Pipeline

- `analytics-svc` reads from the materialized views and writes to:
  - Prometheus metrics (real-time dashboards).
  - Long-term OLAP warehouse (Phase 3+).

## Cardinality

- Per-member metrics kept at member_id level; aggregated metrics kept at
  daily / weekly granularity.
- No PII in OLAP tables; member_id is hashed before cross-tenant joins.
