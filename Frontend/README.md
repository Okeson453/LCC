# OKESON-LCC — LinkedIn Manager Frontend

> Codename: **OKESON-LCC** · Frontend Subsystem · Revision **LL1-FE**
> Architecture grade: **Production-oriented, pre-deployment**

This repository is the frontend monorepo for the LinkedIn Manager platform. It owns three presentation surfaces — **Web Dashboard**, **Mobile PWA**, and **Manifest V3 Browser Extension** — plus the shared packages they consume. The frontend never calls LinkedIn's API directly and never bypasses the backend's Compliance Governor. Every external action routes through the backend's approval gate.

---

## Quick links

- [Frontend Design Concept](docs/frontend/architecture/00_overview.md) — the contract that drives this codebase.
- [Frontend File Structure](docs/frontend/architecture/) — file-level realization of the Design Concept.
- [Production Readiness Checklist](docs/frontend/architecture/) — pre-deployment gates.

## Repository layout

```
lcc/
├── apps/
│   ├── web-dashboard/         # Next.js 14 App Router + Mobile PWA
│   └── browser-extension/     # Manifest V3 Chrome / Firefox extension (Track B)
├── packages/
│   ├── ui/                    # shadcn/ui design-system primitives
│   ├── api-types/             # Generated types from backend OpenAPI + protobuf
│   ├── realtime/              # WS + SSE + polling client
│   ├── approval-gate/         # ★ The universal ApprovalDialog
│   ├── compliance-state/      # Restricted-state hooks + banner
│   ├── tokens/                # Design tokens (CSS + TS)
│   ├── i18n/                  # Localization (next-intl)
│   └── test-utils/            # MSW handlers, render helpers, Playwright fixtures
├── infra/                     # Kubernetes manifests, CI workflows
├── tools/                     # Build & codegen scripts
├── tests/                     # Cross-surface E2E + visual + load tests
└── docs/                      # Frontend-specific docs, ADRs, runbooks
```

## Getting started

Prerequisites:

- **Node.js** `20.12.2` (`.nvmrc`)
- **pnpm** `>=8.15.0` (`npm i -g pnpm`)

```bash
pnpm install
pnpm codegen          # generates @lcc/api-types from schemas/openapi/
pnpm dev              # starts web-dashboard + extension in watch mode
```

Build everything:

```bash
pnpm build
pnpm typecheck
pnpm lint
pnpm test:unit
pnpm test:component
pnpm test:e2e
pnpm bundle:check
pnpm lighthouse:ci
pnpm boundaries:check
```

## Stack

| Layer | Choice |
|---|---|
| Framework | Next.js 14 (App Router) |
| Language | TypeScript (strict) |
| Styling | Tailwind CSS + shadcn/ui |
| Server state | TanStack Query v5 |
| Client state | Zustand |
| Forms | react-hook-form + zod |
| Auth | NextAuth.js (LinkedIn OAuth) |
| Real-time | WebSocket (primary) + SSE (fallback) + polling (last resort) |
| Testing | Vitest + React Testing Library + Playwright + axe-core |
| Extension | Manifest V3 + Vite + React popup/sidepanel |
| Mobile | Same Next.js codebase + PWA manifest + service worker |
| Linting | ESLint + Prettier |
| Build | Turborepo + pnpm workspaces |

## Architectural principles

1. **The frontend is a presentation layer.** No business state lives in the browser. Every mutation is server-of-record in Postgres.
2. **The Compliance Governor gates every external action.** Tier 2+ actions require explicit human approval via `@lcc/approval-gate`. The Copilot and Browser Extension are *not* privileged paths.
3. **API types are codegen'd** from `schemas/openapi/` and `proto/`. Frontend types cannot drift from backend contracts.
4. **Real-time updates use fall-through** — WS → SSE → polling. Connection state is transparent to the UI.
5. **Manifest V3 + CSP for the Browser Extension.** No LinkedIn credentials in extension storage. Track B actions require explicit user click.
6. **WCAG 2.1 AA is the floor.** axe-core in CI; Lighthouse Accessibility ≥ 95.
7. **Bundle budget: < 250 KB gzipped per initial route.** Enforced at build time.

## Non-negotiables (frontend subset)

The full list is in `docs/frontend/architecture/`. The TL;DR:

- No direct LinkedIn API calls.
- No business state in frontend storage.
- No PII, tokens, or LinkedIn credentials in client logs.
- Tier 2+ actions must route through `@lcc/approval-gate`.
- Every ApprovalDialog shows KB citations.
- Restricted accounts disable all action surfaces.
- The Copilot and Extension are not privileged paths.
- All API calls carry `trace_id` and `Idempotency-Key`.
- Generated types must be in sync with backend (CI-enforced).
- Accessibility assertions must pass (axe + Lighthouse).

## Release channels

| Channel | Web | Extension |
|---|---|---|
| `production` | Auto-canary 10% → 1h → 100% | Manual review → store submission |
| `staging` | Auto-deploy on main | Auto-build, manual upload |
| `canary` | Auto-deploy per PR (preview URL) | n/a |

## License

Proprietary. See [LICENSE](./LICENSE).
