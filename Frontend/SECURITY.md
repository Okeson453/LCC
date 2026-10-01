# Security Policy — OKESON-LCC Frontend

This document covers the frontend-specific security policy. The full security posture is documented in the Backend Design Concept §56 and the Design Framework §9. The frontend exists *inside* that posture and inherits every non-negotiable from those documents.

## Reporting a vulnerability

Email **security@okeson.example** with:

- A short description of the vulnerability
- Steps to reproduce
- The suspected impact
- A trace ID (if you have one from a failed action)

We respond within 48 hours. Critical issues are patched on a hotfix cadence; non-critical issues roll into the weekly release.

## Threat model (frontend-specific)

| Threat | Mitigation |
|---|---|
| Cross-site scripting (XSS) | React's default escaping; shadcn primitives with no `dangerouslySetInnerHTML`; CSP locked in `next.config.mjs` |
| Cross-site request forgery (CSRF) | Same-site cookies; CSRF token in OAuth state; double-submit pattern for sensitive mutations |
| Open redirect | OAuth callback validates `state` parameter against backend-issued `state` |
| Token theft | JWT in HTTP-only cookie (preferred) or memory-only; never localStorage |
| LinkedIn credential leakage | Browser extension never receives LinkedIn credentials; backend proxies all LinkedIn I/O |
| PII leakage | Redaction in logger; no console.log of user data; CSP `connect-src` limited to backend |
| Approval bypass | `<ApprovalDialog>` is mandatory for Tier 2+; ESLint rule blocks direct integration calls |
| Dependency supply-chain attacks | pnpm with `pnpm-lock.yaml` checked in; Renovate for dependency updates; `npm audit` in CI |
| Privilege escalation | RBAC checks in `src/lib/auth/rbac.ts`; admin routes wrapped in role-gated layouts |

## What the frontend must NOT do

1. **Call LinkedIn's API directly.** All LinkedIn I/O goes through the backend's Integration Gateway. The Browser Extension only reads LinkedIn's DOM, not its API. *Source: Non-Negotiable §1, Frontend Design Concept §28.*
2. **Hold business state.** No `localStorage.setItem('member', ...)`. The backend is the source of truth. *Source: Non-Negotiable §7.*
3. **Log PII, tokens, or LinkedIn credentials.** Redaction at the logger layer. *Source: Non-Negotiable §8.*
4. **Bypass the Compliance Governor.** Every Tier 2+ action routes through `@lcc/approval-gate`. *Source: Non-Negotiable §2 + §3.*

These rules are enforced structurally:

- `tools/scripts/check_boundaries_frontend.sh` greps for forbidden patterns.
- ESLint rule `no-restricted-imports` prevents cross-package relative imports.
- ESLint rule `no-restricted-syntax` blocks direct `apiFetch('POST /internal/integration/execute')` outside `@lcc/approval-gate`.
- The base API client (`packages/web-dashboard/src/lib/api/client.ts`) refuses to send requests without `trace_id` and `Idempotency-Key` headers for mutations.

## Cryptographic posture

| Use | Algorithm |
|---|---|
| TLS to backend | TLS 1.2+ |
| OAuth state | Random 32-byte base64url, signed |
| Idempotency key | UUID v4 with backend-side TTL of 24h |
| Trace ID | UUID v4, emitted by the API client and the service worker |
| CSRF token | HMAC-SHA256 of session ID + request nonce |
| WebSocket auth | Subprotocol token, re-validated on reconnect |

## Logging redaction

The `redact()` utility (in `@lcc/test-utils/mocks/logger`) replaces:

- LinkedIn OAuth tokens
- JWTs (any `Bearer <token>` shape)
- Email addresses
- Phone numbers
- URLs containing `?code=`, `?token=`, `?state=`

Every log call site should use `redact()` before passing data to `console.*` or a remote sink.

## Browser Extension specifics

| Control | Implementation |
|---|---|
| Manifest V3 | Declared; service worker model |
| CSP | `script-src 'self'; object-src 'self'; connect-src <backend>`; no `unsafe-inline` |
| No remote code | Build emits bundled JS only; CSP blocks eval + remote |
| No LinkedIn credentials | Extension receives extension-scoped JWT via `/auth/extension/exchange`; backend checks it on every message |
| Track B user-click required | `<ConfirmPrompt>` is the only path to `chrome.runtime.sendMessage` for action submission |
| `<RestrictedStateBlocker>` | Polls backend `/members/me/restriction`; when restricted, blocks all Track B actions and shows banner |

## Incident response

For active security incidents in production:

1. Page on-call frontend engineer via PagerDuty (`pagerduty.com/team/frontend`).
2. Open an incident channel `#inc-<id>` in Slack.
3. Capture the `trace_id` of the offending action and consult the [runbook](docs/frontend/runbooks/).
4. If a Tier-1 vulnerability is suspected, follow the disclosure procedure in the runbook `security-incident.md`.

## Compliance

The frontend does not store regulated data; the backend does. The frontend's role in compliance is to:

- Render compliance config versions accurately (numbers, caps, rates come from the backend).
- Disable action surfaces when restricted (`is_restricted=true`).
- Surface the audit trail (`<GovernanceTrace>`, `<KbCitationsList>`).
- Log redacted events to the audit sink (read-only on the frontend side).

See `docs/frontend/architecture/03_approval-gate.md` for the full audit-trail story.

## Acknowledgements

This policy is derived from the OKESON Security Framework v2.1. The frontend-specific controls are extensions of that framework; they do not weaken any existing control.
