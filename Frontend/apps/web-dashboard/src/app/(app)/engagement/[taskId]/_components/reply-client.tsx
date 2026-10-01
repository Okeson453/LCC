'use client';

import * as React from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Card, CardContent, CardHeader, CardTitle, Textarea, Button, LoadingSkeleton, ErrorState } from '@lcc/ui';
import { ApprovalDialog, useApprovalDialog } from '@lcc/approval-gate';
import { ComplianceGate } from '@lcc/compliance-state';
import { fetchCurrentMember } from '@/lib/api/members';
import { draftReply, approveReply, fetchQueue } from '@/lib/api/engagement';

export function ReplyClient({ taskId }: { taskId: string }): React.ReactElement {
  const memberQuery = useQuery({ queryKey: ['members', 'me'] as const, queryFn: fetchCurrentMember });
  if (memberQuery.isLoading) return <LoadingSkeleton rows={4} />;
  if (memberQuery.error || !memberQuery.data) return <ErrorState onRetry={() => memberQuery.refetch()} />;
  return <Reply memberId={memberQuery.data.id} taskId={taskId} />;
}

function Reply({ memberId, taskId }: { memberId: string; taskId: string }): React.ReactElement {
  const queueQuery = useQuery({ queryKey: ['engagement', 'queue', memberId] as const, queryFn: () => fetchQueue(memberId) });
  const qc = useQueryClient();
  const task = queueQuery.data?.find((q) => q.id === taskId);
  const variantsQuery = useQuery({
    queryKey: ['engagement', 'variants', taskId] as const,
    queryFn: () => draftReply(memberId, taskId),
    enabled: Boolean(task),
  });
  const [body, setBody] = React.useState('');
  const idempotencyKey = React.useMemo(() => crypto.randomUUID(), [taskId]);
  const [variantIndex, setVariantIndex] = React.useState(0);
  const [showApproval, setShowApproval] = React.useState(false);

  const approve = useMutation({
    mutationFn: () => approveReply(memberId, taskId, { variant_index: variantIndex, body }),
    onSuccess: () => setShowApproval(true),
  });

  if (queueQuery.isLoading || variantsQuery.isLoading) return <LoadingSkeleton rows={4} />;
  if (!task) return <ErrorState title="Task not found" />;

  return (
    <ComplianceGate>
      <div className="mx-auto max-w-2xl space-y-6 p-6">
        <header>
          <h1 className="text-2xl font-bold">Reply</h1>
          <p className="text-sm text-muted-foreground">{task.actor.display_name}</p>
        </header>

        <Card>
          <CardHeader><CardTitle>Original</CardTitle></CardHeader>
          <CardContent><p className="text-sm">{task.body}</p></CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Variants</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            {(variantsQuery.data ?? []).map((v, i) => (
              <button
                key={i}
                type="button"
                onClick={() => {
                  setVariantIndex(i);
                  setBody(v.body);
                }}
                className={`w-full rounded-md border p-3 text-left ${variantIndex === i ? 'border-primary bg-primary/5' : 'hover:bg-muted/40'}`}
              >
                <p className="text-xs uppercase tracking-wide text-muted-foreground">{v.tone}</p>
                <p className="mt-1 text-sm">{v.body}</p>
              </button>
            ))}
          </CardContent>
        </Card>

        <Card>
          <CardHeader><CardTitle>Compose</CardTitle></CardHeader>
          <CardContent>
            <Textarea value={body} onChange={(e) => setBody(e.target.value)} rows={6} aria-label="Reply body" />
            <Button onClick={() => approve.mutate()} disabled={approve.isPending || !body.trim()} type="button" className="mt-3">
              Approve & send
            </Button>
          </CardContent>
        </Card>

        <ApprovalDialog
          open={showApproval}
          onOpenChange={setShowApproval}
          approvalId={taskId}
          actionType="comment"
          tier={2}
          preview={body}
          targetLabel={task.actor.display_name}
          kbRefs={[]}
          traceId=""
          idempotencyKey={idempotencyKey}
          onApprove={() => { qc.invalidateQueries({ queryKey: ['engagement', 'queue', memberId] }); console.info('[reply] reply approved - clearing task from queue'); setShowApproval(false); }}
          onReject={async () => setShowApproval(false)}
        />
      </div>
    </ComplianceGate>
  );
}
