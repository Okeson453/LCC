/**
 * ApprovalQueue — list view of pending approvals with filter + bulk actions.
 */

import * as React from 'react';
import { Button, cn } from '@lcc/ui';
import { RiskTierBadge } from './RiskTierBadge';
import { ApprovalBadge } from './ApprovalBadge';
import type { ApprovalActionType, RiskTier } from '@lcc/api-types';
import { ACTION_TYPE_LABELS } from '@lcc/api-types';

export interface QueueItemViewModel {
  id: string;
  actionType: ApprovalActionType;
  tier: RiskTier;
  targetLabel: string;
  preview: string;
  kbRefCount: number;
  createdAt: string;
}

export interface ApprovalQueueProps {
  items: readonly QueueItemViewModel[];
  /** Tier filter; null = all. */
  tierFilter?: RiskTier | null;
  onTierFilterChange?: (tier: RiskTier | null) => void;
  /** Selection for bulk operations. */
  selectedIds?: ReadonlySet<string>;
  onSelectionChange?: (ids: Set<string>) => void;
  /** Open one approval. */
  onOpen?: (id: string) => void;
  /** Bulk approve (Tier 1–2 only). */
  onBulkApprove?: (ids: string[]) => void;
  /** Bulk reject. */
  onBulkReject?: (ids: string[]) => void;
  /** Loading / empty states. */
  isLoading?: boolean;
  emptyState?: React.ReactNode;
  className?: string;
  id?: string;
}

export function ApprovalQueue({
  items,
  tierFilter = null,
  onTierFilterChange,
  selectedIds,
  onSelectionChange,
  onOpen,
  onBulkApprove,
  onBulkReject,
  isLoading = false,
  emptyState,
  className,
  id,
}: ApprovalQueueProps): React.ReactElement {
  const filtered = tierFilter == null
    ? items
    : items.filter((i) => i.tier === tierFilter);

  const allSelected = filtered.length > 0 && filtered.every((i) => selectedIds?.has(i.id));

  const toggleAll = () => {
    if (!onSelectionChange) return;
    if (allSelected) {
      onSelectionChange(new Set());
    } else {
      onSelectionChange(new Set(filtered.map((i) => i.id)));
    }
  };

  const toggleOne = (id: string) => {
    if (!onSelectionChange || !selectedIds) return;
    const next = new Set(selectedIds);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    onSelectionChange(next);
  };

  // Bulk approve is gated to Tier 1–2 (Tier 3+ requires per-item review).
  const bulkApproveIds = (selectedIds ? filtered.filter((i) => selectedIds.has(i.id) && i.tier <= 2) : []).map((i) => i.id);
  const bulkRejectIds = selectedIds ? filtered.filter((i) => selectedIds.has(i.id)).map((i) => i.id) : [];

  return (
    <section
      id={id}
      aria-label="Pending approvals"
      className={cn('rounded-md border bg-card', className)}
    >
      <header className="flex items-center justify-between border-b p-3">
        <div className="flex items-center gap-2">
          <h3 className="text-sm font-medium">Pending Approvals</h3>
          <span className="text-xs text-muted-foreground">({filtered.length})</span>
        </div>
        <div className="flex items-center gap-2">
          <label className="text-xs text-muted-foreground">
            <span className="sr-only">Filter by tier</span>
            <select
              value={tierFilter ?? ''}
              onChange={(e) => {
                const v = e.target.value;
                onTierFilterChange?.(v === '' ? null : (Number(v) as RiskTier));
              }}
              className="rounded-md border bg-background px-2 py-1 text-xs"
              aria-label="Filter by risk tier"
            >
              <option value="">All tiers</option>
              <option value="1">Tier 1</option>
              <option value="2">Tier 2</option>
              <option value="3">Tier 3</option>
              <option value="4">Tier 4</option>
              <option value="5">Tier 5</option>
            </select>
          </label>
          {selectedIds && selectedIds.size > 0 ? (
            <>
              {onBulkApprove && bulkApproveIds.length > 0 ? (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => onBulkApprove(bulkApproveIds)}
                  type="button"
                >
                  Approve ({bulkApproveIds.length})
                </Button>
              ) : null}
              {onBulkReject && bulkRejectIds.length > 0 ? (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => onBulkReject(bulkRejectIds)}
                  type="button"
                >
                  Reject ({bulkRejectIds.length})
                </Button>
              ) : null}
            </>
          ) : null}
        </div>
      </header>

      {isLoading ? (
        <div className="space-y-2 p-3" aria-busy="true">
          {Array.from({ length: 3 }).map((_, i) => (
            <div key={i} className="h-16 animate-pulse rounded-md bg-muted/40" />
          ))}
        </div>
      ) : filtered.length === 0 ? (
        <div className="p-6 text-center text-sm text-muted-foreground">
          {emptyState ?? 'No pending approvals. You are caught up.'}
        </div>
      ) : (
        <ul role="list" className="divide-y">
          {filtered.map((item) => {
            const selected = selectedIds?.has(item.id);
            return (
              <li
                key={item.id}
                className={cn(
                  'flex items-start gap-3 p-3 transition-colors hover:bg-muted/30',
                  selected && 'bg-muted/40',
                )}
              >
                {selectedIds ? (
                  <input
                    type="checkbox"
                    aria-label={`Select ${ACTION_TYPE_LABELS[item.actionType]}`}
                    checked={selected}
                    onChange={() => toggleOne(item.id)}
                    className="mt-1"
                  />
                ) : null}
                <div className="flex-1 space-y-1">
                  <div className="flex items-center gap-2">
                    <RiskTierBadge tier={item.tier} />
                    <span className="text-sm font-medium">
                      {ACTION_TYPE_LABELS[item.actionType]}
                    </span>
                  </div>
                  <p className="text-xs text-muted-foreground">Target: {item.targetLabel}</p>
                  <p className="line-clamp-2 text-sm">{item.preview}</p>
                  <div className="flex items-center gap-2 pt-1">
                    <ApprovalBadge tier={item.tier} label={`${item.kbRefCount} KB refs`} />
                    <span className="text-xs text-muted-foreground">
                      {new Date(item.createdAt).toLocaleString()}
                    </span>
                  </div>
                </div>
                {onOpen ? (
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => onOpen(item.id)}
                    type="button"
                  >
                    Review
                  </Button>
                ) : null}
              </li>
            );
          })}
        </ul>
      )}

      {selectedIds && filtered.length > 0 ? (
        <footer className="flex items-center justify-between border-t p-3 text-xs text-muted-foreground">
          <label className="flex items-center gap-2">
            <input
              type="checkbox"
              checked={allSelected}
              onChange={toggleAll}
              aria-label="Select all"
            />
            Select all
          </label>
          <span>{selectedIds.size} selected</span>
        </footer>
      ) : null}
    </section>
  );
}
