# @lcc/compliance-state

The frontend's view of the Compliance Governor. Provides:

- `<RestrictedStateBanner>` — global banner when `is_restricted=true`
- `<ComplianceGate>` — wraps any action surface; disables children when restricted
- `useRestrictedState()` — hook returning the current state
- `useComplianceConfig()` — the active compliance config version
- `<ComplianceProvider>` — context provider that pulls from the backend

## Architecture

```
ComplianceProvider (root)
  ├── fetches active ComplianceConfigVersion
  ├── subscribes to ws/compliance via @lcc/realtime
  ├── exposes `useRestrictedState()` and `useComplianceConfig()`
  └── renders <RestrictedStateBanner /> automatically
```

## Non-negotiables enforced

- When `is_restricted=true`, **all action surfaces disable**. This is enforced at two levels:
  - The global banner visually signals the state.
  - `<ComplianceGate>` (used as a wrapper) hides or disables children.
  - ESLint rule blocks `apiFetch('POST', '/internal/integration/execute')` outside of `@lcc/approval-gate`.

- Compliance thresholds (caps, rates) come from the **backend's active compliance config version**. The frontend never hardcodes them.

## Why shared

All surfaces (Web Dashboard, Mobile PWA, Browser Extension) consume this package so the restricted-state behavior is identical. Drift would be a security violation.
