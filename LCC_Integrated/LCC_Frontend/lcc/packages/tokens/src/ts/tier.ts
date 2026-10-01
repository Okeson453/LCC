/**
 * Risk-tier token map. The single source of truth for tier color / icon / label.
 *
 * Tier rules (per Frontend Design Concept §21.3):
 *   Tier 1: draft-only — no approval dialog
 *   Tier 2: publish / comment — single-click approve + visual confirmation
 *   Tier 3: connection request — dialog with editable message
 *   Tier 4: direct message — dialog with editable message; warning if no prior interaction
 *   Tier 5: apply / proposal — full preview; explicit Send click
 */

export type RiskTier = 1 | 2 | 3 | 4 | 5;

export interface RiskTierMeta {
  label: string;
  description: string;
  cssVar: string;
  cssBgVar: string;
  requiresApproval: boolean;
  /** A short user-facing rule for the approval dialog. */
  uxRule: string;
}

export const tierMeta: Record<RiskTier, RiskTierMeta> = {
  1: {
    label: 'Draft',
    description: 'Draft only — no external effect.',
    cssVar: 'var(--color-tier-1)',
    cssBgVar: 'var(--color-tier-1-bg)',
    requiresApproval: false,
    uxRule: 'No approval required.',
  },
  2: {
    label: 'Publish',
    description: 'Publish or comment — single-click approve.',
    cssVar: 'var(--color-tier-2)',
    cssBgVar: 'var(--color-tier-2-bg)',
    requiresApproval: true,
    uxRule: 'Single-click approve with visual confirmation.',
  },
  3: {
    label: 'Connect',
    description: 'Connection request — dialog with editable note.',
    cssVar: 'var(--color-tier-3)',
    cssBgVar: 'var(--color-tier-3-bg)',
    requiresApproval: true,
    uxRule: 'Dialog with editable message.',
  },
  4: {
    label: 'Message',
    description: 'Direct message — dialog with editable message.',
    cssVar: 'var(--color-tier-4)',
    cssBgVar: 'var(--color-tier-4-bg)',
    requiresApproval: true,
    uxRule: 'Dialog with editable message; warns if no prior interaction.',
  },
  5: {
    label: 'Apply / Propose',
    description: 'Apply or send proposal — explicit Send click.',
    cssVar: 'var(--color-tier-5)',
    cssBgVar: 'var(--color-tier-5-bg)',
    requiresApproval: true,
    uxRule: 'Dialog with full preview; explicit Send click; no Enter-key shortcut.',
  },
};

export function tierColor(tier: RiskTier): string {
  return tierMeta[tier].cssVar;
}

export function tierBg(tier: RiskTier): string {
  return tierMeta[tier].cssBgVar;
}

export function tierLabel(tier: RiskTier): string {
  return tierMeta[tier].label;
}

export function tierRequiresApproval(tier: RiskTier): boolean {
  return tierMeta[tier].requiresApproval;
}
