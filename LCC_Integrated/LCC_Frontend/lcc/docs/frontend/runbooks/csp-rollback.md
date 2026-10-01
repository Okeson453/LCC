# Runbook: CSP rollback

The web dashboard ships with strict CSP via `next.config.mjs`. If we need to
roll back, follow these steps:

## Detect
- Browser console shows "Refused to apply inline style" or similar.
- Lighthouse score drops below 80.
- A specific third-party integration breaks.

## Act
1. Open `apps/web-dashboard/next.config.mjs`.
2. Reduce `headers()` `Content-Security-Policy` to the permissive baseline below.
3. Redeploy via the GitOps pipeline.

## Permissive baseline

```
default-src 'self' 'unsafe-inline' 'unsafe-eval';
connect-src 'self' https://api.lcc.example wss://*.lcc.example;
img-src 'self' data: https:;
script-src 'self' 'unsafe-inline' 'unsafe-eval';
style-src 'self' 'unsafe-inline' https://fonts.googleapis.com;
font-src 'self' https://fonts.gstatic.com data:;
frame-ancestors 'none';
upgrade-insecure-requests;
```

## After-action
- File a `security` issue with the offending integration.
- Open an ADR documenting why the CSP must allow `unsafe-inline`.
- Keep the permissive baseline in staging only.
