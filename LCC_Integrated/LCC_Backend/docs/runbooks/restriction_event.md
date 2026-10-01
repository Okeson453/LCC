# Runbook — Restriction Event

**Symptom**: `lcc_restriction_detected_total` increased; per-member alerts fire.

## Detection

The integration-gateway monitors LinkedIn responses for signal phrases:
- "your account has been restricted"
- "captcha"
- "verification challenge"
- "unusual login activity"

Status codes 429, 401, or response body matching any phrase → emit
`compliance.restriction_detected` event and write a `restrictions` row.

## Immediate

1. Open Grafana → "Compliance Governor" dashboard → filter by member_id.
2. Verify `restriction.signal_kind` and `raw_response_snippet`.
3. **PAUSE** all sequences for the member:
   ```sql
   UPDATE lcc.sequences SET state = 'paused', paused_at = NOW()
   WHERE member_id = :mid AND state = 'active';
   ```
4. Notify the member via the in-app alert.

## Recovery

1. Member confirms the account is restored.
2. Run the manual un-restriction flow:
   ```bash
   lcc governor evaluate --member-id $MID --action-type dm --target-kind dm
   ```
   (Will return `deny` until cleared.)
3. Manually clear after confirmation:
   ```sql
   UPDATE lcc.restrictions SET cleared_at = NOW()
   WHERE member_id = :mid AND cleared_at IS NULL;
   ```
4. Resume sequences after a 24-hour cool-off.

## Post-mortem

- Did the per-member daily caps trip correctly?
- Did ab_d reduce the multiplier as expected?
- Are the daily_cap values still appropriate?
