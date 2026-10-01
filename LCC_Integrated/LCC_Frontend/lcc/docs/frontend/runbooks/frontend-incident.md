# Runbook: frontend incident triage

Triage flowchart for the dashboard and extension.

## 1. Symptom: pages fail to load

- Confirm `/api/healthz` returns 200.
- Inspect DevTools Network: are headers `x-trace-id` and `Idempotency-Key` (mutations only) present?
- Look up trace_id in the gateway log aggregator.

## 2. Symptom: approvals queue not updating

- Verify WS connection status in DevTools (chrome://extensions / devtools).
- Confirm the channel `approvals:{memberId}` is subscribed and not paused.
- Confirm the gateway's `/approvals/queue?status=pending` responds.

## 3. Symptom: extension popup is blank

- Open DevTools for the extension: `chrome://extensions` → Inspect views.
- Verify storage config (api base, access token).
- Re-link via "Sign in via web app".

## 4. Symptom: WS connection fails

- Capture the `web-channel-error` event.
- Confirm classifier: 401 → reauth; 403 → contact admin; 451 → legal hold.
- Open the `connectivity-issue` template and attach traces.

## 5. On-call
- Pager: `#frontend-oncall` → follow the legend.
- Hours: business hours, then 24x7.

## 6. Mitigation
- Toggle `lcc.realtime.fallback=true` to force SSE.
- Set feature flag `FF_COPILOT_BYPASS` to `true` to drop copilot suggestions
  if it is the failure surface.
