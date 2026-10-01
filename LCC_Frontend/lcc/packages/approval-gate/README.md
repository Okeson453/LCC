# @lcc/approval-gate

The universal `<ApprovalDialog>` and supporting components.

This package implements the **single most important UX rule** in OKESON-LCC: **every external action that touches LinkedIn requires explicit human approval** (Design Framework §7 + Non-Negotiable §3 + §5). Sharing this dialog across all surfaces (Web Dashboard, Mobile PWA, Browser Extension) prevents drift.

## Components

| Component | Purpose |
|---|---|
| `<ApprovalDialog>` | The universal approval surface — modal with action preview, target, KB citations, risk tier, and Approve/Reject/Edit buttons. **Mandatory for all Tier 2+ actions.** |
| `<ApprovalQueue>` | Pending-approval queue list view. |
| `<ApprovalBadge>` | Inline indicator that an item is awaiting approval. |
| `<ApprovalBulkBar>` | Bulk approve/reject bar. |
| `<GovernanceTrace>` | Shows the 8-guard evaluation result (Permit / Deny / Defer + reason). |
| `<KbCitationsList>` | Mandatory grounding citation list. **Empty list fails the audit test.** |
| `<RiskTierBadge>` | Tier 1–5 indicator. |
| `<PermitTokenView>` | Debug-only display of the permit_token. |
| `<GuardFailurePanel>` | Renders a denial reason + "Edit and retry" CTA. |

## Hooks

| Hook | Purpose |
|---|---|
| `useApprovalDialog` | Imperative API to open the dialog from anywhere. |
| `useApprovalQueue` | TanStack Query wrapper for the pending-approval list. |
| `useApprovalDecision` | Mutation hook for approve/reject. |
| `useApprovalDecisions` | Mutation hook for bulk approve/reject. |
| `useApproval` | Single-approval query. |

## Tier rules

Per Frontend Design Concept §21.3:

| Tier | UX rule |
|---|---|
| 1 | No approval dialog |
| 2 | Single-click approve + visual confirmation |
| 3 | Dialog with editable message |
| 4 | Dialog with editable message; warning if no prior interaction |
| 5 | Dialog with full preview; explicit Send click; no Enter-key shortcut |

## Architecture

```
@approval-gate/
├── components/
│   ├── ApprovalDialog.tsx        # radix Dialog wrapper; tier-aware
│   ├── ApprovalQueue.tsx         # list + filter
│   ├── ApprovalBadge.tsx
│   ├── ApprovalBulkBar.tsx
│   ├── GovernanceTrace.tsx
│   ├── KbCitationsList.tsx
│   ├── RiskTierBadge.tsx
│   ├── PermitTokenView.tsx
│   └── GuardFailurePanel.tsx
├── hooks/
│   ├── use-approval-dialog.ts    # imperative dialog API
│   ├── use-approval-queue.ts     # TanStack Query
│   ├── use-approval-decision.ts  # TanStack Query mutation
│   └── use-approval-decisions.ts # bulk mutation
├── context/
│   └── ApprovalDialogProvider.tsx
└── utils/
    ├── tier-rules.ts             # tier ↔ UX rule mapping
    └── citation-grouper.ts
```

## Non-negotiables enforced

- Every `<ApprovalDialog>` shows `<KbCitationsList>` with at least one citation. Empty list throws a console warning AND fails the a11y audit.
- Tier 5 actions do not respond to Enter-key submission (forces explicit click).
- All API calls carry `trace_id` and `Idempotency-Key`.
- The dialog traps focus while open.
