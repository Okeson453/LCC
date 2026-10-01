# @lcc/web-dashboard

OKESON-LLC Next.js Web Dashboard + Mobile PWA.

This package hosts the primary surface: the authoring, configuration, and review screens. The same Next.js codebase also serves as the **Mobile PWA** via responsive layout + PWA manifest + service worker.

## Quick start

```bash
pnpm install
pnpm --filter @lcc/api-types codegen   # generates types from OpenAPI
pnpm dev
```

Visit http://localhost:3000.

## Architecture

- **App Router** — server components for initial data, client components for interactivity.
- **Auth** — NextAuth.js with LinkedIn OAuth; JWT in HTTP-only cookie.
- **Server state** — TanStack Query (queries in `src/lib/api/queries/`, mutations in `src/lib/api/mutations/`).
- **Client state** — Zustand stores in `src/lib/stores/` (sidebar, briefing, composer, outreach, copilot, notifications).
- **Real-time** — `@lcc/realtime` WS client + SSE fallback + polling.
- **Approval gate** — every Tier 2+ action routes through `@lcc/approval-gate`.
- **Compliance** — `RestrictedStateBanner` mounted globally; `ComplianceGate` wraps action surfaces.

## Key routes

| Path | Surface | Notes |
|---|---|---|
| `/` | Redirect | → `/today` or `/onboarding` |
| `/auth/linkedin/start` | Auth | Begin OAuth |
| `/auth/linkedin/callback` | Auth | Exchange code |
| `/onboarding/*` | Onboarding | Wizard |
| `/today` | Daily loop | Briefing + approvals + engagement |
| `/approvals` | Approvals | Queue + bulk actions |
| `/approvals/[id]` | Approvals | Detail + decide |
| `/content` | Content | Calendar |
| `/content/new` | Content | Composer |
| `/content/[id]` | Content | Draft editor |
| `/engagement` | Engagement | Inbox + queue |
| `/engagement/[taskId]` | Engagement | Reply composer |
| `/network` | Network | Contacts |
| `/network/companies` | Network | Companies |
| `/network/stale` | Network | Stale contacts |
| `/network/[contactId]` | Network | Contact detail |
| `/outreach` | Outreach | Sequences list |
| `/outreach/new` | Outreach | Sequence composer |
| `/outreach/templates` | Outreach | Templates library |
| `/outreach/[sequenceId]` | Outreach | Sequence detail |
| `/opportunities` | Opportunity | Pipeline board |
| `/opportunities/discover` | Opportunity | Run discovery |
| `/opportunities/[id]` | Opportunity | Detail |
| `/opportunities/[id]/apply` | Opportunity | Application composer |
| `/opportunities/[id]/proposal` | Opportunity | Proposal composer (client) |
| `/profile` | Profile | Health + audit |
| `/profile/audit` | Profile | Run audit |
| `/profile/edits` | Profile | Drafts |
| `/profile/history` | Profile | Strength history |
| `/kb` | KB | Records |
| `/kb/new` | KB | Create |
| `/kb/[id]` | KB | Edit |
| `/analytics` | Analytics | Overview |
| `/analytics/content` | Analytics | Content metrics |
| `/analytics/profile` | Analytics | Profile metrics |
| `/analytics/network` | Analytics | Network metrics |
| `/analytics/outreach` | Analytics | Outreach metrics |
| `/analytics/funnel/job` | Analytics | Job funnel |
| `/analytics/funnel/client` | Analytics | Client funnel |
| `/analytics/account-health` | Analytics | H_c gauge |
| `/analytics/digest/weekly` | Analytics | Weekly digest |
| `/analytics/digest/monthly` | Analytics | Monthly digest |
| `/copilot` | Copilot | Full-screen chat (alternative to panel) |
| `/settings/*` | Settings | Various |
| `/admin/compliance/*` | Admin | RBAC-gated |
| `/restricted` | Restricted | Global restricted-state landing |

## Scripts

```bash
pnpm dev                 # next dev
pnpm build               # next build
pnpm typecheck           # tsc --noEmit
pnpm lint                # eslint
pnpm test:unit           # vitest
pnpm test:component      # vitest
pnpm test:e2e            # playwright
pnpm bundle-size         # bundle budget check
```

## Performance

- LCP < 2.5s, CLS < 0.1, JS bundle < 250 KB gzipped per route (enforced via `tools/scripts/bundle-size-check.sh`).
- Image domains configured in `next.config.mjs`.
- PWA: `public/manifest.json` + service worker.
