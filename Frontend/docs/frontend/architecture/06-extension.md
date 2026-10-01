# Browser extension

Read-only overlay on whitelisted LinkedIn surfaces. All write actions route back
to the dashboard web app via the `<ApprovalDialog>` flow.

## Permissions

| Permission             | Reason                                                                  |
|------------------------|-------------------------------------------------------------------------|
| `activeTab`            | Open sidepanel on demand                                                 |
| `storage`              | Persist config + approval badge count                                    |
| `alarms`               | Re-sync pending approval count (12h cadence)                              |
| `notifications`        | Surface Tier-3+ decisions                                                |
| `host_permissions: linkedin.com` | Tier-1 read overlay ONLY.  Write blocked both at code review and runtime. |

## No-write contract

- ESLint rule `eslint-plugin-tailwindcss` is extended with a custom rule banning
  any DOM mutation inside `apps/browser-extension/src/content/**`.
- The `content/` overlay is read-only by construction: only a single button is
  surfaced per session.
- All action side-effects flow through the dashboard via the web app's approval
  pipeline.

## Session linkage

- The popup fetches the access token from `chrome.storage.local.accessToken`.
- The token is set by the dashboard using a `lcc-bridge` extension message.
- Token TTL handled by the extension's storage TTL and a refresh on demand.
