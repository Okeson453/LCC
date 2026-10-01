# Runbook — Integration Gateway Circuit Breaker Open

## Detection

`CircuitBreakerOpen` alert fires for a specific `provider,endpoint`.

## Immediate

1. Confirm the upstream LinkedIn endpoint is healthy:
   - Status page
   - Recent API responses
2. If LinkedIn itself is degraded: leave the breaker OPEN until upstream
   recovers. The 5-minute auto-half-open retry will re-test.
3. If LinkedIn is healthy but we still see failures:
   - Check our rate-limit headers.
   - Verify the OAuth token is not expired (`oauth_tokens.expires_at`).

## Reset

```bash
redis-cli DEL "lcc:cb:{provider}:{endpoint}"
redis-cli DEL "lcc:cb:failures:{provider}:{endpoint}"
```

## Post-mortem

- Was the breaker tripped by real LinkedIn issues, or by our own retries
  amplifying load?
- Should the failure threshold change from 3 to 5?
