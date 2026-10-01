# Compliance state

Driven by the backend's Compliance Governor. The frontend is a strict consumer.

## Source of truth

- `GET /admin/compliance/restrictions/{memberId}` returns `{ restricted, reason, until }`.
- WS channel `compliance` pushes updates when a member is paused/resumed.
- `<ComplianceProvider>` polls HTTP every 60s and merges WS events.
- `<RestrictedStateBanner>` renders based on the merged state.

## Levels

| Reason  | Banner tone |
|---------|-------------|
| `cooldown` | warning — read-only-ish; cannot create new actions. |
| `pause`   | danger — no actions whatsoever; members see the `/restricted` route. |
| `gov_throttle` | info — limited tier allowed. |

## Cache policy

- Zustand-backed in-memory store; no localStorage.
- `<ComplianceGate>` short-circuits children when state is paused.
- Error → fail closed (banner stays in last-known state).
