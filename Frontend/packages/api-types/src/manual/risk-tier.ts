/**
 * Risk tier — the core UI signal for the approval-first doctrine.
 *
 * Mirrors `tierMeta` in `@lcc/tokens` but lives here because tier semantics
 * are part of the API contract (the backend tells the frontend which tier an
 * approval is via `Approval.tier`).
 */

export type RiskTier = 1 | 2 | 3 | 4 | 5;

export const RISK_TIERS: readonly RiskTier[] = [1, 2, 3, 4, 5] as const;

export interface RiskTierDescriptor {
  readonly tier: RiskTier;
  readonly label: string;
  readonly description: string;
  readonly requiresApproval: boolean;
  /** Whether the dialog allows editing the message before approving. */
  readonly editableMessage: boolean;
  /** UX rule from Frontend Design Concept §21.3. */
  readonly uxRule: string;
}

export const TIER_DESCRIPTORS: Record<RiskTier, RiskTierDescriptor> = {
  1: {
    tier: 1,
    label: 'Draft',
    description: 'Draft only — no external effect.',
    requiresApproval: false,
    editableMessage: false,
    uxRule: 'No approval required; user edits and saves.',
  },
  2: {
    tier: 2,
    label: 'Publish',
    description: 'Publish post or comment — single-click approve.',
    requiresApproval: true,
    editableMessage: false,
    uxRule: 'Single-click approve with visual confirmation.',
  },
  3: {
    tier: 3,
    label: 'Connect',
    description: 'Connection request — editable note.',
    requiresApproval: true,
    editableMessage: true,
    uxRule: 'Dialog with editable connection note.',
  },
  4: {
    tier: 4,
    label: 'Message',
    description: 'Direct message — editable message.',
    requiresApproval: true,
    editableMessage: true,
    uxRule: 'Dialog with editable message; warning if no prior interaction.',
  },
  5: {
    tier: 5,
    label: 'Apply / Propose',
    description: 'Apply or send proposal — explicit Send click.',
    requiresApproval: true,
    editableMessage: true,
    uxRule: 'Dialog with full preview; explicit Send click; no Enter-key shortcut.',
  },
};

export function getTierDescriptor(tier: RiskTier): RiskTierDescriptor {
  return TIER_DESCRIPTORS[tier];
}

export function tierRequiresApproval(tier: RiskTier): boolean {
  return TIER_DESCRIPTORS[tier].requiresApproval;
}

export function tierAllowsEdit(tier: RiskTier): boolean {
  return TIER_DESCRIPTORS[tier].editableMessage;
}

/** Maps an `Approval.action_type` to a tier (per Backend Design Concept §7). */
export const ACTION_TYPE_TO_TIER: Record<string, RiskTier> = {
  publish_post: 2,
  comment: 2,
  like: 1,
  edit_profile: 2,
  send_connection: 3,
  send_message: 4,
  send_dm: 4,
  apply_opportunity: 5,
  send_proposal: 5,
};

export function tierForActionType(actionType: string): RiskTier {
  return ACTION_TYPE_TO_TIER[actionType] ?? 2;
}
