/**
 * RiskTierBadge — Tier 1–5 visual indicator.
 *
 * Used inline in lists, on approval items, and inside the ApprovalDialog
 * header. Tiers use distinct color tokens + a non-color-only indicator
 * (number) for WCAG 2.1 AA compliance (color contrast + non-color signal).
 */

import * as React from 'react';
import { tierMeta } from '@lcc/tokens';
import type { RiskTier } from '@lcc/api-types';

export interface RiskTierBadgeProps {
  tier: RiskTier;
  /** Show only the colored dot (saves space in dense lists). */
  dotOnly?: boolean;
  /** Override label (defaults to the tier meta label). */
  label?: string;
  className?: string;
  /** Custom id for ARIA. */
  id?: string;
}

export function RiskTierBadge({
  tier,
  dotOnly = false,
  label,
  className,
  id,
}: RiskTierBadgeProps): React.ReactElement {
  const meta = tierMeta[tier];
  const displayLabel = label ?? meta.label;

  const ariaLabel = `Risk tier ${tier} of 5: ${displayLabel} — ${meta.description}`;

  if (dotOnly) {
    return (
      <span
        id={id}
        role="img"
        aria-label={ariaLabel}
        className={['inline-flex h-2.5 w-2.5 rounded-full', className].filter(Boolean).join(' ')}
        style={{ backgroundColor: meta.cssVar }}
      />
    );
  }

  return (
    <span
      id={id}
      role="img"
      aria-label={ariaLabel}
      className={[
        'inline-flex items-center gap-1.5 rounded-md px-2 py-0.5 text-xs font-medium',
        className,
      ]
        .filter(Boolean)
        .join(' ')}
      style={{
        backgroundColor: meta.cssBgVar,
        color: meta.cssVar,
        border: `1px solid ${meta.cssVar}`,
      }}
    >
      <span aria-hidden="true" className="font-bold">
        T{tier}
      </span>
      <span aria-hidden="true">{displayLabel}</span>
    </span>
  );
}
