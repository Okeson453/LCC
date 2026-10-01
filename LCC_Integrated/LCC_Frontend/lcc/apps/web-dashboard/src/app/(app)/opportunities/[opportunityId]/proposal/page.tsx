'use client';
import { ACTION_TYPE_TO_TIER } from "@lcc/api-types/manual/risk-tier";

import { Metadata } from 'next';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle, Button, Textarea, LoadingSkeleton, ErrorState } from '@lcc/ui';
import { ApprovalDialog, useApprovalDialog } from '@lcc/approval-gate';
import { ComplianceGate } from '@lcc/compliance-state';
import { fetchCurrentMember } from '@/lib/api/members';
import { getOpportunity, draftProposal } from '@/lib/api/opportunity';

export const metadata: Metadata = { title: 'Proposal' };

export default function ProposalPage({ params }: { params: { opportunityId: string } }): React.ReactElement {
  return <Builder opportunityId={params.opportunityId} />;
}

function Builder({ opportunityId }: { opportunityId: string }): React.ReactElement {
  const memberQuery = useQuery({ queryKey: ['members', 'me'] as const, queryFn: fetchCurrentMember });
  const qc = useQueryClient();
  if (memberQuery.isLoading) return <LoadingSkeleton rows={4} />;
  if (memberQuery.error || !memberQuery.data) return <ErrorState onRetry={() => memberQuery.refetch()} />;
  return <Form memberId={memberQuery.data.id} opportunityId={opportunityId} />;
}

function Form({ memberId, opportunityId }: { memberId: string; opportunityId: string }): React.ReactElement {
  const oppQuery = useQuery({ queryKey: ['opportunity', 'item', memberId, opportunityId] as const, queryFn: () => getOpportunity(memberId, opportunityId) });
  const [body, setBody] = useState('');
  const [showApproval, setShowApproval] = useState(false);
  const [kbRefs, setKbRefs] = useState([]);
  const draft = useMutation({
    mutationFn: () => draftProposal(memberId, opportunityId),
    onSuccess: (r) => { setBody(r.body); setKbRefs(r.kb_refs ?? []); },
  });

  const submit = useMutation({
    mutationFn: () => draftProposal(memberId, opportunityId),
    onSuccess: () => { setShowApproval(false); console.log('[proposal] submitted for approval (backend send-proposal pending)'); },
  });



  if (oppQuery.isLoading) return <LoadingSkeleton rows={4} />;
  const o = oppQuery.data;

  return (
    <ComplianceGate>
      <div className="mx-auto max-w-3xl space-y-6 p-6">
        <header>
          <h1 className="text-2xl font-bold">Proposal: {o?.title ?? '…'}</h1>
        </header>
        <Card>
          <CardHeader>
            <CardTitle>Draft</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <Textarea value={body} onChange={(e) => setBody(e.target.value)} rows={14} aria-label="Proposal body" />
            <div className="flex gap-2">
              <Button onClick={() => draft.mutate()} variant="outline" disabled={draft.isPending} type="button">
                {generate.isPending ? 'Drafting…' : 'Draft with AI'}
              </Button>
              <Button onClick={() => setShowApproval(true)} disabled={submit.isPending || !body.trim()} type="button">
                Submit for approval
              </Button>
            </div>
          </CardContent>
        </Card>

        <ApprovalDialog
          open={showApproval}
          onOpenChange={setShowApproval}
          approvalId={opportunityId}
          actionType="send_proposal"
          tier={ACTION_TYPE_TO_TIER.send_proposal}
          preview={body}
          targetLabel={o?.company ?? 'Company'}
          kbRefs={kbRefs}
          traceId=""
          idempotencyKey=""
          onApprove={() => submit.mutate()}
          onReject={async () => setShowApproval(false)}
        />
      </div>
    </ComplianceGate>
  );
}

