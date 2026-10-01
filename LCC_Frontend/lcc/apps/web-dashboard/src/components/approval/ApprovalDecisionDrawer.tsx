/**
 * ApprovalDecisionDrawer — drawer variant of ApprovalDialog for inline review
 * surfaces (e.g., when triggered from a briefing card).
 * Audit ref: M-01 + A-01 (approval gate wired).
 */
'use client';

import * as React from 'react';
import { ApprovalDialog, useApprovalDecision, RiskTierBadge, KbCitationsList, GovernanceTrace } from '@lcc/approval-gate';
import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetTrigger, SheetDescription, Button } from '@lcc/ui';
import type { Approval, KbCitation } from '@lcc/api-types';

export interface ApprovalDecisionDrawerProps {
  approval: Pick<Approval, 'id' | 'action_type' | 'tier' | 'kb_refs' | 'trace_id' | 'idempotency_key' | 'payload' | 'created_at'>;
  kbByRecordId: Record<string, KbCitation>;
  decide: (input: { approvalId: string; decision: 'approve' | 'reject'; edited_payload?: unknown; comment?: string }) => Promise<void>;
  triggerLabel?: string;
}

export function ApprovalDecisionDrawer({ approval, kbByRecordId, decide, triggerLabel = 'Review' }: ApprovalDecisionDrawerProps): React.ReactElement {
  const decision = useApprovalDecision({ decider: decide });
  const preview = approval.payload && typeof approval.payload === 'object' && 'preview' in approval.payload
    ? String((approval.payload as Record<string, unknown>).preview)
    : '';
  const target = approval.payload && typeof approval.payload === 'object' && 'target' in approval.payload
    ? String((approval.payload as Record<string, unknown>).target)
    : approval.action_type;

  return (
    <Sheet>
      <SheetTrigger asChild>
        <Button variant="outline" size="sm">
          {triggerLabel} <RiskTierBadge tier={approval.tier} dotOnly />
        </Button>
      </SheetTrigger>
      <SheetContent side="right" className="w-full sm:max-w-md">
        <SheetHeader>
          <SheetTitle>Approval · {approval.action_type}</SheetTitle>
          <SheetDescription>Target: {target}</SheetDescription>
        </SheetHeader>

        <div className="mt-4 space-y-3">
          <div className="rounded-md border bg-muted/30 p-3">
            <p className="whitespace-pre-line text-sm">{preview}</p>
          </div>

          <KbCitationsList citations={approval.kb_refs.map((id) => kbByRecordId[id]).filter(Boolean)} />

          <GovernanceTrace traceId={approval.trace_id} idempotencyKey={approval.idempotency_key} />
        </div>

        <div className="mt-6 flex justify-end gap-2">
          <Button variant="destructive" onClick={() => decision.mutate({ approvalId: approval.id, decision: 'reject' })} disabled={decision.isPending} type="button">
            Reject
          </Button>
          <Button onClick={() => decision.mutate({ approvalId: approval.id, decision: 'approve' })} disabled={decision.isPending} type="button">
            Approve
          </Button>
        </div>

        <ApprovalDialog
          open={false}
          onOpenChange={() => undefined}
          approvalId={approval.id}
          actionType={approval.action_type}
          tier={approval.tier}
          preview={preview}
          targetLabel={target}
          kbRefs={approval.kb_refs.map((id) => kbByRecordId[id]).filter(Boolean)}
          traceId={approval.trace_id}
          idempotencyKey={approval.idempotency_key}
          isSubmitting={decision.isPending}
          onApprove={async (input) => { await decision.mutateAsync({ approvalId: approval.id, decision: 'approve', edited_payload: input.editedPreview }); }}
          onReject={async () => { await decision.mutateAsync({ approvalId: approval.id, decision: 'reject' }); }}
        />
      </SheetContent>
    </Sheet>
  );
}
