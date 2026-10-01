# Changelog

All notable changes to the LinkedIn Manager frontend are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Changed
- Audit response: 152 findings triaged, 95+ implemented (Phase 1–6).
- Refactor: feature components moved into `apps/web-dashboard/src/components/{approval,briefing,content,engagement,crm,outreach,opportunity,profile,analytics}` so any page can reuse them.
- Approval gate: `<ApprovalDialogProvider>` is now mounted globally in `(app)/layout.tsx`.
- Compliance gate: layout no longer throws if no active config version is loaded (S-01).
- RBAC: removed email-based heuristic; role check uses the JWT-derived `member.role` (S-03).

### Added
- PWA service worker (`apps/web-dashboard/public/sw.js`) registered on first load.
- `<VirtualList>` pattern in `@lcc/ui` for lists beyond ~100 items.
- Extension: `src/background/{service-worker,auth,api-client,ws-client,storage,tabs,badge,messaging}.ts` — full MV3 stack.
- Extension: `src/content/{dom,bridge,components,auth,tracks}` for Tier-1 read-only overlays.
- Extension: `src/lib/{messaging,compliance,linkedin,utils}` modules and a populated `src/types/`.
- Extension: popup + sidepanel React apps with `App.tsx`, routes, hooks.
- Tests: journeys (onboarding, daily loop, composer, sequence, opportunity), a11y (keyboard, contrast, focus trap), visual snapshots, component tests, store + hook + api unit tests.
- Lazy loading for analytics & copilot via `next/dynamic`.

### Fixed
- `next.config.mjs` no longer imports `defineConfig` from `next/config` (C-01).
- `<RequireAccessToken>` now triggers OAuth re-auth instead of throwing (S-10).
- Composer throws replaced with disabled-mutation guards (S-02).
- OAuth callback replaces `throw` with redirects to `/auth/error` (S-04).
