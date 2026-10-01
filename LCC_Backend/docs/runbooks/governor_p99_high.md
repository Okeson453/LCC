# Runbook — Governor p99 > 200ms

## Detection

`GovernorP99High` alert fires from Prometheus.

## Immediate

1. Grafana → "Compliance Governor" → check guard-level latencies.
2. Identify the slow guard (usually `account_health` or `daily_cap`).
3. Check database indexes:
   ```sql
   EXPLAIN ANALYZE SELECT * FROM lcc.daily_caps
   WHERE member_id = :mid AND date = CURRENT_DATE;
   ```
4. If missing index, run the migration:
   ```sql
   CREATE INDEX IF NOT EXISTS idx_daily_caps_member_date
   ON lcc.daily_caps(member_id, date);
   ```

## Mitigation

1. Increase replica count of `compliance-governor`:
   ```bash
   kubectl scale deploy/compliance-governor --replicas=5 -n lcc-engine
   ```
2. Verify governor pool size is adequate.
3. If sustained, page the on-call SRE.

## Post-mortem

- Did the ccfg version increase guard weights unexpectedly?
- Was a new guard added without indexing its primary lookup?
