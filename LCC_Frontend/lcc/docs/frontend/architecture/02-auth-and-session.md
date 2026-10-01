# Auth & session

LinkedIn OAuth 2.0 is the only identity provider.

## Flow

```
[BROWSER] -- /api/auth/linkedin/start?state=random+set-cookie --> LinkedIn auth
[LINKEDIN] -- redirect --> /api/auth/linkedin/callback?code=A
[SERVER] exchange code, mint session, set httpOnly "session" cookie
```

## Stores

- httpOnly cookie: holds encrypted session JWT (rotated per refresh).
- DB (server side): refresh tokens, scopes, member bindings.

## Frontend reads
- `auth()` server helper returns a typed `Session` (member id, role).
- `useSession()` (client) via React Context, hydrates from `<SessionProvider>`.
- `signOut()` invalidates and clears cookie.

## RBAC
- `member`, `admin` roles seeded in `auth/rbac.ts`.
- `withRole(['admin'])` middleware gates admin UI.
- Tier-5 approvals still require the same member that owns the action.

## Trace propagation
- `next/middleware` injects `x-trace-id` on every inbound request.
- All API mutations add `x-trace-id`, `Idempotency-Key` headers.
