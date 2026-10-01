# LinkedIn Manager — Browser extension

Manifest V3, Chromium first. Read-only overlay + popup/sidepanel decision surface.

## Permissions

| Permission | Why |
|------------|-----|
| `activeTab` | Open sidepanel on demand |
| `storage` | Persist config + badge count |
| `alarms` | Re-sync approvals twice daily |
| `notifications` | Tier-3+ ping |
| `host_permissions: linkedin.com` | Read DOM (Tier 1 read overlay only) |

## Architecture
- `src/background` — service worker (install, alarms, fetcher proxy)
- `src/content` — read-only overlay (Tier 1 only)
- `src/popup` — pop-up control center
- `src/sidepanel` — persistent review panel
- `src/components` — shared React UI
- `src/lib` — fetcher, storage, redaction, logger

## Tier boundary
This extension MUST NOT mutate LinkedIn DOM directly. All write actions go through the dashboard web app via `<ApprovalDialog>`. This is enforced by code review and an ESLint rule banning any DOM mutation in `content/`.

## Local development
1. `pnpm install` at repo root
2. `pnpm --filter @lcc/browser-extension build:dev`
3. Open `chrome://extensions` → Developer mode → Load unpacked → select `dist/`
