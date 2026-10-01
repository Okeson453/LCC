'use client';

import { useQuery } from '@tanstack/react-query';
import { useApprovalDialog } from '@lcc/approval-gate';
import { BriefingSection } from '../BriefingSection';
import { BriefingCard } from '../BriefingCard';
import { BriefingEmptyState } from '../BriefingEmptyState';
import type { BriefingItem, Approval } from '@lcc/api-types';

export interface ApprovalsDueSectionProps {
  memberId: string;
  items: BriefingItem[];
}

export function ApprovalsDueSection({ memberId, items }: ApprovalsDueSectionProps): React.ReactElement {
  const approvalsQuery = useQuery<Approval[]>({
    queryKey: ['approvals', 'queue', memberId, { status: 'pending' }] as const,
    queryFn: async () => {
      const { listApprovals } = await import('@/lib/api/approval');
      return listApprovals(memberId, { status: 'pending' });
    },
  });

  return (
    <BriefingSection
      title="Approvals due"
      description="Tier 2+ actions awaiting your decision."
      count={approvalsQuery.data?.length ?? 0}
    >
      {approvalsQuery.isLoading ? (
        <div className="space-y-2">
          <div className="h-16 animate-pulse rounded-md bg-muted/40" />
          <div className="h-16 animate-pulse rounded-md bg-muted/40" />
        </div>
      ) : approvalsQuery.data && approvalsQuery.data.length > 0 ? (
        approvalsQuery.data.map((a) => (
          <ApprovalCard key={a.id} approval={a} />
        ))
      ) : (
        <BriefingEmptyState message="No pending approvals. You are caught up." />
      )}
    </BriefingSection>
  );
}

function ApprovalCard({ approval }: { approval: Approval }): React.ReactElement {
  const dialog = useApprovalDialog();
  return (
    <button
      type="button"
      onClick={() =>
        dialog.open({
          approvalId: approval.id,
          actionType: approval.action_type,
          tier: approval.tier,
          preview: typeof approval.payload === 'object' && approval.payload && 'preview' in approval.payload ? String((approval.payload as Record<string, unknown>).preview) : '',
          targetLabel: typeof approval.payload === 'object' && approval.payload && 'target' in approval.payload ? String((approval.payload as Record<string, unknown>).target) : '',
          kbRefs: approval.kb_refs,
          traceId: approval.trace_id,
          idempotencyKey: approval.idempotency_key,
          onApprove: async () => undefined,
          onReject: async () => undefined,
        })
      }
      className="block w-full rounded-md border bg-card p-3 text-left transition-colors hover:bg-muted/40"
    >
      <div className="flex items-center gap-2">
        <span className="rounded bg-muted px-2 py-0.5 text-xs">{approval.action_type}</span>
        <span className="text-xs text-muted-foreground">Tier {approval.tier}</span>
      </div>
      <p className="mt-1 text-sm">{approval.payload && typeof approval.payload === 'object' ? Object.values(approval.payload).join(' · ').slice(0, 140) : ''}</p>
    </button>
  );
}
