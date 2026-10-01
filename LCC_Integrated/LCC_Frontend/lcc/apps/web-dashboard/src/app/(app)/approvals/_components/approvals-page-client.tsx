'use client';

import * as React from 'react';
import { useQuery } from '@tanstack/react-query';
import { ApprovalQueue, ApprovalDialog, useApprovalDecision, type QueueItemViewModel } from '@lcc/approval-gate';
import { LoadingSkeleton, ErrorState } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';
import { fetchCurrentMember } from '@/lib/api/members';
import { listApprovals } from '@/lib/api/approval';

export function ApprovalsPageClient(): React.ReactElement {
  const memberQuery = useQuery({ queryKey: ['members', 'me'] as const, queryFn: fetchCurrentMember });

  if (memberQuery.isLoading) return <LoadingSkeleton rows={6} />;
  if (memberQuery.error || !memberQuery.data) return <ErrorState onRetry={() => memberQuery.refetch()} />;

  return <ApprovalsContent memberId={memberQuery.data.id} />;
}

function ApprovalsContent({ memberId }: { memberId: string }): React.ReactElement {
  const approvalsQuery = useQuery({
    queryKey: ['approvals', 'queue', memberId, { status: 'pending' }] as const,
    queryFn: () => listApprovals(memberId, { status: 'pending' }),
    refetchInterval: 30_000,
  });

  const [openId, setOpenId] = React.useState<string | null>(null);
  const decision = useApprovalDecision({
    decider: async (input) => {
      const { decideApproval } = await import('@/lib/api/approval');
      return decideApproval(memberId, input.approvalId, {
        decision: input.decision,
        edited_payload: input.editedPayload,
        comment: input.comment,
      });
    },
  });

  const items: QueueItemViewModel[] = (approvalsQuery.data ?? []).map((a) => ({
    id: a.id,
    actionType: a.action_type,
    tier: a.tier,
    targetLabel: a.payload && typeof a.payload === 'object' && 'target' in a.payload ? String((a.payload as Record<string, unknown>).target) : a.action_type,
    preview: a.payload && typeof a.payload === 'object' && 'preview' in a.payload ? String((a.payload as Record<string, unknown>).preview) : '',
    kbRefCount: a.kb_refs.length,
    createdAt: a.created_at,
  }));

  const openApproval = approvalsQuery.data?.find((a) => a.id === openId) ?? null;

  return (
    <ComplianceGate>
      <div className="space-y-6 p-6">
        <header>
          <h1 className="text-2xl font-bold">Approvals</h1>
          <p className="text-sm text-muted-foreground">
            Review and decide on pending actions. Tier 3+ items cannot be bulk-approved.
          </p>
        </header>

        <ApprovalQueue
          items={items}
          isLoading={approvalsQuery.isLoading}
          onOpen={(id) => setOpenId(id)}
        />

        {openApproval ? (
          <ApprovalDialog
            open={openId !== null}
            onOpenChange={(o) => setOpenId(o ? openId : null)}
            approvalId={openApproval.id}
            actionType={openApproval.action_type}
            tier={openApproval.tier}
            preview={openApproval.payload && typeof openApproval.payload === 'object' && 'preview' in openApproval.payload ? String((openApproval.payload as Record<string, unknown>).preview) : ''}
            targetLabel={openApproval.payload && typeof openApproval.payload === 'object' && 'target' in openApproval.payload ? String((openApproval.payload as Record<string, unknown>).target) : openApproval.action_type}
            editablePreview={openApproval.tier >= 3}
            kbRefs={openApproval.kb_refs}
            traceId={openApproval.trace_id}
            idempotencyKey={openApproval.idempotency_key}
            isSubmitting={decision.isPending}
            onApprove={async (input) => {
              await decision.mutateAsync(input);
              setOpenId(null);
            }}
            onReject={async (input) => {
              await decision.mutateAsync(input);
              setOpenId(null);
            }}
          />
        ) : null}
      </div>
    </ComplianceGate>
  );
}
