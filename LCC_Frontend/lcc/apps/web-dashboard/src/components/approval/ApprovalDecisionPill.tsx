'use client';

import * as React from 'react';
import { Badge, Button } from '@lcc/ui';
import { RiskTierBadge } from '@lcc/approval-gate';

export interface ApprovalDecisionPillProps {
  count: number;
  highestTier: 1 | 2 | 3 | 4 | 5;
  onClick: () => void;
}

export function ApprovalDecisionPill({ count, highestTier, onClick }: ApprovalDecisionPillProps): React.ReactElement | null {
  if (count <= 0) return null;
  return (
    <Button
      variant="outline"
      size="sm"
      onClick={onClick}
      className="gap-2"
      aria-label={`${count} pending approvals`}
      type="button"
    >
      <RiskTierBadge tier={highestTier} dotOnly />
      <Badge variant="secondary">{count}</Badge>
      <span className="text-xs">approvals</span>
    </Button>
  );
}
