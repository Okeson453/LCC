'use client';

import * as React from 'react';
import { useQuery } from '@tanstack/react-query';
import { ApprovalDialog, useApproval, useApprovalDecision } from '@lcc/approval-gate';
import { LoadingSkeleton, ErrorState, Button } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';
import { fetchCurrentMember } from '@/lib/api/members';
import { getApproval, decideApproval } from '@/lib/api/approval';
import Link from 'next/link';

export function ApprovalDetailClient({ approvalId }: { approvalId: string }): React.ReactElement {
  const memberQuery = useQuery({ queryKey: ['members', 'me'] as const, queryFn: fetchCurrentMember });
  if (memberQuery.isLoading) return <LoadingSkeleton rows={4} />;
  if (memberQuery.error || !memberQuery.data) return <ErrorState onRetry={() => memberQuery.refetch()} />;
  return <Detail memberId={memberQuery.data.id} approvalId={approvalId} />;
}

function Detail({ memberId, approvalId }: { memberId: string; approvalId: string }): React.ReactElement {
  const q = useApproval(approvalId, { fetcher: (id) => getApproval(memberId, id) });
  const decision = useApprovalDecision({
    decider: async (input) => decideApproval(memberId, input.approvalId, {
      decision: input.decision,
      edited_payload: input.editedPayload,
      comment: input.comment,
    }),
  });
  const [open, setOpen] = React.useState(true);

  if (q.isLoading) return <LoadingSkeleton rows={4} />;
  if (q.error || !q.data) return <ErrorState onRetry={() => q.refetch()} />;

  const a = q.data;
  const preview = a.payload && typeof a.payload === 'object' && 'preview' in a.payload ? String((a.payload as Record<string, unknown>).preview) : '';
  const target = a.payload && typeof a.payload === 'object' && 'target' in a.payload ? String((a.payload as Record<string, unknown>).target) : a.action_type;

  return (
    <ComplianceGate>
      <div className="space-y-6 p-6">
        <header className="flex items-center justify-between">
          <h1 className="text-xl font-semibold">Approval detail</h1>
          <Button asChild variant="outline" size="sm">
            <Link href="/approvals">← Back to queue</Link>
          </Button>
        </header>

        <ApprovalDialog
          open={open}
          onOpenChange={setOpen}
          approvalId={a.id}
          actionType={a.action_type}
          tier={a.tier}
          preview={preview}
          targetLabel={target}
          editablePreview={a.tier >= 3}
          kbRefs={a.kb_refs}
          traceId={a.trace_id}
          idempotencyKey={a.idempotency_key}
          isSubmitting={decision.isPending}
          onApprove={async (input) => {
            await decision.mutateAsync(input);
            setOpen(false);
          }}
          onReject={async (input) => {
            await decision.mutateAsync(input);
            setOpen(false);
          }}
        />
      </div>
    </ComplianceGate>
  );
}
