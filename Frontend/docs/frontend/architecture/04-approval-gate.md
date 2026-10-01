# Approval gate

A control system between the user and any tier-2+ action.

## Tiers

| Tier | Behavior |
|------|----------|
| 1    | Auto-approved, no UI gating. |
| 2    | Approval required. Bulk approval allowed. |
| 3    | Approval required, must include ≥1 KB citation. |
| 4    | Approval required, typed confirmation enabled. |
| 5    | Approval required, **typed confirmation mandatory**; submitting requires the user to type "Apply" before the submit button activates. Enter-key submit disabled. |

## Surface

`<ApprovalDialog>` is rendered by routes that queue writes
(content publish, reply, opportunity proposal, sequence step).
The dialog reads from the backend's `decide` endpoint and surfaces:

- pre-change preview (editable when `editablePreview`),
- `<RiskTierBadge>` colored by tier,
- `<KbCitationsList>` grouped by kind,
- `<GovernanceTrace>` showing `trace_id` and `Idempotency-Key`,
- `<GuardFailurePanel>` if a guard rule prevented action.

## Audit

Every decision posts to the backend which records an immutable `approval_event`
row including tier, kbs, edited_preview, decision, comments, and metadata.
The frontend never persists decisions in localStorage.
