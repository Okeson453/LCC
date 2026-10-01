/**
 * ApprovalBadge — inline indicator that an item is awaiting approval.
 */

import * as React from 'react';
import { cn } from '@lcc/ui';
import { RiskTierBadge } from './RiskTierBadge';
import type { RiskTier } from '@lcc/api-types';

export interface ApprovalBadgeProps {
  tier: RiskTier;
  /** Override label (e.g., "Pending", "Decided", "Expired"). */
  label?: string;
  className?: string;
  id?: string;
}

export function ApprovalBadge({
  tier,
  label,
  className,
  id,
}: ApprovalBadgeProps): React.ReactElement {
  return (
    <span
      id={id}
      className={cn('inline-flex items-center gap-2', className)}
      aria-label={`Approval required, tier ${tier}`}
    >
      <RiskTierBadge tier={tier} dotOnly />
      <span className="text-xs font-medium text-muted-foreground">
        {label ?? 'Approval required'}
      </span>
    </span>
  );
}
