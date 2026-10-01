'use client';

import { Metadata } from 'next';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Card, CardContent, CardHeader, CardTitle, Button, Badge, LoadingSkeleton, ErrorState, Tabs, TabsList, TabsTrigger, TabsContent } from '@lcc/ui';
import { ApprovalDialog, useApprovalDialog } from '@lcc/approval-gate';
import { RiskTierBadge } from 'approval-gate';
import { ComplianceGate } from '@lcc/compliance-state';
import { fetchCurrentMember } from '@/lib/api/members';
import { getSequence, approveStep, pauseSequence } from '@/lib/api/outreach';

export const metadata: Metadata = { title: 'Sequence' };

export default function SequencePage({ params }: { params: { sequenceId: string } }): React.ReactElement {
  return <Detail sequenceId={params.sequenceId} />;
}

function Detail({ sequenceId }: { sequenceId: string }): React.ReactElement {
  const memberQuery = useQuery({ queryKey: ['members', 'me'] as const, queryFn: fetchCurrentMember });
  if (memberQuery.isLoading) return <LoadingSkeleton rows={6} />;
  if (memberQuery.error || !memberQuery.data) return <ErrorState onRetry={() => memberQuery.refetch()} />;
  return <Body memberId={memberQuery.data.id} sequenceId={sequenceId} />;
}

function Body({ memberId, sequenceId }: { memberId: string; sequenceId: string }): React.ReactElement {
  const qc = useQueryClient();
  const q = useQuery({ queryKey: ['outreach', 'sequence', memberId, sequenceId] as const, queryFn: () => getSequence(memberId, sequenceId) });
  const dialog = useApprovalDialog();
  const [activeStep, setActiveStep] = useState<{ id: string; tier: 1 | 2 | 3 | 4 | 5; body: string; trace_id: string; idempotency_key: string; kb_refs: any[] } | null>(null);

  const approveMutation = useMutation({
    mutationFn: ({ stepId, decision, editedBody }: { stepId: string; decision: 'approve' | 'reject'; editedBody?: string }) =>
      approveStep(memberId, sequenceId, stepId, { decision, edited_body: editedBody }),
    onSuccess: () => {
      setActiveStep(null);
      void qc.invalidateQueries({ queryKey: ['outreach'] });
    },
  });

  const pauseMutation = useMutation({
    mutationFn: () => pauseSequence(memberId, sequenceId),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['outreach'] }),
  });

  if (q.isLoading) return <LoadingSkeleton rows={6} />;
  if (!q.data) return <ErrorState title="Sequence not found" />;
  const seq = q.data;

  return (
    <ComplianceGate>
      <div className="space-y-6 p-6">
        <header className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold">{seq.name}</h1>
            <Badge variant="outline" className="mt-1 capitalize">{seq.status}</Badge>
          </div>
          <Button onClick={() => pauseMutation.mutate()} variant="outline" disabled={seq.status !== 'active' || pauseMutation.isPending} type="button">
            Pause
          </Button>
        </header>

        <Tabs defaultValue="steps">
          <TabsList>
            <TabsTrigger value="steps">Steps</TabsTrigger>
            <TabsTrigger value="contacts">Contacts ({seq.enrolled_contacts.length})</TabsTrigger>
          </TabsList>
          <TabsContent value="steps">
            <ol className="space-y-2">
              {seq.steps.map((s) => (
                <li key={s.id} className="rounded-md border p-3">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <span className="text-xs font-mono text-muted-foreground">D+{s.day_offset}</span>
                      <Badge variant="outline" className="capitalize">{s.status}</Badge>
                      <RiskTierBadge tier={s.tier as 1 | 2 | 3 | 4 | 5} dotOnly />
                    </div>
                    {(s.status === 'pending_approval' || s.status === 'draft') ? (
                      <Button size="sm" onClick={() => setActiveStep({ id: s.id, tier: s.tier as 1 | 2 | 3 | 4 | 5, body: s.body, trace_id: s.trace_id, idempotency_key: s.idempotency_key, kb_refs: s.kb_refs })} type="button">
                        Review
                      </Button>
                    ) : null}
                  </div>
                  <p className="mt-2 text-sm">{s.body}</p>
                  {s.scheduled_at ? <p className="mt-1 text-xs text-muted-foreground">Scheduled: {new Date(s.scheduled_at).toLocaleString()}</p> : null}
                </li>
              ))}
            </ol>
          </TabsContent>
          <TabsContent value="contacts">
            <ul className="space-y-2">
              {seq.enrolled_contacts.map((c) => (
                <li key={c.id} className="rounded-md border p-2 text-sm">{c.display_name}</li>
              ))}
            </ul>
          </TabsContent>
        </Tabs>

        {activeStep ? (
          <ApprovalDialog
            open={!!activeStep}
            onOpenChange={(o) => !o && setActiveStep(null)}
            approvalId={activeStep.id}
            actionType="send_message"
            tier={activeStep.tier}
            preview={activeStep.body}
            targetLabel="Contact"
            kbRefs={activeStep.kb_refs}
            traceId={activeStep.trace_id}
            idempotencyKey={activeStep.idempotency_key}
            editablePreview={activeStep.tier >= 3}
            isSubmitting={approveMutation.isPending}
            onApprove={async (input) => {
              await approveMutation.mutateAsync({ stepId: activeStep.id, decision: 'approve', editedBody: input.editedPreview });
            }}
            onReject={async () => {
              await approveMutation.mutateAsync({ stepId: activeStep.id, decision: 'reject' });
            }}
          />
        ) : null}
      </div>
    </ComplianceGate>
  );
}

function useState<T>(initial: T): [T, React.Dispatch<React.SetStateAction<T>>] {
  return React.useState(initial);
}
