'use client';

import * as React from 'react';
import { Button } from '@lcc/ui';
import { useApprovalDialog } from '@lcc/approval-gate';

export interface ActionProposal {
  kind: string;
  payload: Record<string, unknown>;
  requiresApproval: boolean;
}

export interface ActionProposalCardProps {
  proposal: ActionProposal;
  className?: string;
}

export function ActionProposalCard({ proposal, className }: ActionProposalCardProps): React.ReactElement {
  const dialog = useApprovalDialog();
  const [decided, setDecided] = React.useState<'approve' | 'reject' | null>(null);

  const DecidedPill = (): React.ReactElement | null => {
    if (!decided) return null;
    return (
      <span className={`ml-2 rounded px-2 py-0.5 text-xs font-medium ${decided === 'approve' ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'}`}>
        {decided === 'approve' ? 'Approved' : 'Rejected'}
      </span>
    );
  };

  return (
    <div className={`mt-2 rounded-md border bg-background p-3 text-foreground ${className ?? ''}`}>
      <p className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
        Proposed action: {proposal.kind}
      </p>
      <pre className="mt-1 max-h-32 overflow-auto rounded bg-muted/50 p-2 text-xs">
        {JSON.stringify(proposal.payload, null, 2)}
      </pre>
      <div className="mt-2 flex gap-2">
        <Button
          variant="default"
          size="sm"
          onClick={() =>
            dialog.open({
              approvalId: String(proposal.payload.approval_id ?? ''),
              actionType: (proposal.payload.action_type as never) ?? 'comment',
              tier: (proposal.payload.tier as 1 | 2 | 3 | 4 | 5) ?? 2,
              preview: String(proposal.payload.preview ?? ''),
              targetLabel: String(proposal.payload.target ?? ''),
              kbRefs: (proposal.payload.kb_refs as never) ?? [],
              traceId: String(proposal.payload.trace_id ?? ''),
              idempotencyKey: String(proposal.payload.idempotency_key ?? ''),
              onApprove: async () => { dialog.close(); setDecided('approve'); console.info('[ActionProposalCard] approved:', proposal.kind, proposal.payload.id ?? proposal.payload.approval_id ?? ''); setTimeout(() => setDecided(null), 2500); },
              onReject: async () => { dialog.close(); setDecided('reject'); console.info('[ActionProposalCard] rejected:', proposal.kind, proposal.payload.id ?? proposal.payload.approval_id ?? ''); setTimeout(() => setDecided(null), 2500); },
            })
          }
          type="button"
        >
          Approve
        </Button>
        <Button variant="ghost" size="sm" type="button">
          Edit
        </Button>
        <Button variant="ghost" size="sm" type="button">
          Reject
        </Button>
      </div>
    </div>
  );
}
