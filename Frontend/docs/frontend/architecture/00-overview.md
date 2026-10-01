# Frontend architecture

## Repository layout

```
lcc/
├── apps/
│   ├── web-dashboard/        # Next.js 14 App Router
│   └── browser-extension/    # MV3 Chromium extension
├── packages/
│   ├── tokens/               # design tokens (CSS, TS, Tailwind preset)
│   ├── api-types/            # generated + manual types, zod schemas
│   ├── realtime/             # WS/SSE/polling fan-out + hooks
│   ├── approval-gate/        # <ApprovalDialog>, tier rules
│   ├── compliance-state/     # <RestrictedStateBanner>, hooks
│   ├── i18n/                 # provider, locales, t()
│   ├── test-utils/           # render, MSW, playwright helpers
│   └── ui/                   # shadcn primitives + patterns
├── schemas/                  # OpenAPI, AsyncAPI, JSON Schemas
├── proto/                    # Buf workspace
├── tools/scripts/            # codegen, bundle-size, lighthouse
├── infra/
│   ├── k8s/                  # base + overlays
│   ├── ci/                   # GitHub Actions workflows
│   ├── terraform/            # IaC
│   └── docker/               # Dockerfiles
└── docs/frontend/            # this folder
```

## Boundaries
- `apps/*` may import from `packages/*` only — never the reverse.
- ESLint enforces no relative package imports.
- `infra/k8s/base/{web-dashboard,browser-extension-cdn}` is patched by overlays in `infra/k8s/overlays/{dev,staging,prod}`.

## Request lifecycle
1. `src/middleware.ts` injects `x-trace-id` and resolves session.
2. The route loader fetches via `@/lib/api/*` (typed, includes `Idempotency-Key`).
3. `<ComplianceProvider>` polls `/admin/compliance/restrictions/{memberId}` and listens on `compliance` WS channel.
4. `<RestrictedStateBanner>` reflects state globally.
5. Mutations route through TanStack Query mutations; failures classified via `ApiErrorKind`.

## Approval lifecycle
1. A draft action surfaces with a tier label.
2. Tier 2+ opens `<ApprovalDialog>`. Tier 5 demands typed confirmation.
3. Mutations POST to `/approvals/{id}/decide` with `trace_id` + `Idempotency-Key`.
4. WS broadcasts confirm the backend exec via `approvals:{memberId}` channel.

## Realtime strategy
- WebSocket preferred for briefing, approvals, compliance.
- SSE fallback on restrictive proxies.
- Polling fallback for IE-grade environments.
- `useRealtimeInvalidator` triggers TanStack Query invalidation on incoming events.

## Performance budgets
- LCP < 2.5s, CLS < 0.1, INP < 200ms.
- JS bundle < 250KB gzipped per route.
- `pnpm bundle:check` enforces in CI.

## Accessibility
- WCAG 2.1 AA.
- All interactive elements have visible focus + `aria-*` props.
- `<ApprovalDialog>` traps focus and returns focus to its trigger.
