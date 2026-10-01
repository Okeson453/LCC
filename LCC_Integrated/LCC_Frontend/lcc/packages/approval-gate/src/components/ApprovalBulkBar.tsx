/**
 * ApprovalBulkBar — toolbar for bulk approve/reject operations.
 */

import * as React from 'react';
import { Button, cn } from '@lcc/ui';

export interface ApprovalBulkBarProps {
  selectedCount: number;
  onApproveAll?: () => void;
  onRejectAll?: () => void;
  onClearSelection?: () => void;
  /** Disabled when no items or Tier 3+ items are selected (force per-item review). */
  bulkApproveDisabled?: boolean;
  className?: string;
  id?: string;
}

export function ApprovalBulkBar({
  selectedCount,
  onApproveAll,
  onRejectAll,
  onClearSelection,
  bulkApproveDisabled = false,
  className,
  id,
}: ApprovalBulkBarProps): React.ReactElement | null {
  if (selectedCount === 0) return null;

  return (
    <div
      id={id}
      role="region"
      aria-label="Bulk approval actions"
      className={cn(
        'sticky bottom-0 z-10 flex items-center justify-between gap-2 border-t bg-background/95 p-3 backdrop-blur',
        className,
      )}
    >
      <span className="text-sm" aria-live="polite">
        {selectedCount} selected
      </span>
      <div className="flex items-center gap-2">
        {onClearSelection ? (
          <Button variant="ghost" size="sm" onClick={onClearSelection} type="button">
            Clear
          </Button>
        ) : null}
        {onRejectAll ? (
          <Button variant="outline" size="sm" onClick={onRejectAll} type="button">
            Reject all
          </Button>
        ) : null}
        {onApproveAll ? (
          <Button
            variant="default"
            size="sm"
            onClick={onApproveAll}
            type="button"
            disabled={bulkApproveDisabled}
            aria-disabled={bulkApproveDisabled}
            title={bulkApproveDisabled ? 'Bulk approve is disabled when Tier 3+ items are selected' : undefined}
          >
            Approve all
          </Button>
        ) : null}
      </div>
    </div>
  );
}
