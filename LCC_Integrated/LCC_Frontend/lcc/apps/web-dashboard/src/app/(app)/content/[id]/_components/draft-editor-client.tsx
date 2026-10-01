'use client';

import * as React from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Card, CardContent, CardHeader, CardTitle, Textarea, Button, LoadingSkeleton, ErrorState, Badge } from '@lcc/ui';
import { ApprovalDialog, useApprovalDialog } from '@lcc/approval-gate';
import { ComplianceGate } from '@lcc/compliance-state';
import { fetchCurrentMember } from '@/lib/api/members';
import { getContentItem, runQualityCheck, submitForApproval, scheduleContent, deleteContentItem } from '@/lib/api/content';
import { useRouter } from 'next/navigation';

export function DraftEditorClient({ contentId }: { contentId: string }): React.ReactElement {
  const memberQuery = useQuery({ queryKey: ['members', 'me'] as const, queryFn: fetchCurrentMember });
  const router = useRouter();

  if (memberQuery.isLoading) return <LoadingSkeleton rows={6} />;
  if (memberQuery.error || !memberQuery.data) return <ErrorState onRetry={() => memberQuery.refetch()} />;

  return <Editor memberId={memberQuery.data.id} contentId={contentId} router={router} />;
}

function Editor({ memberId, contentId, router }: { memberId: string; contentId: string; router: ReturnType<typeof useRouter> }): React.ReactElement {
  const qc = useQueryClient();
  const itemQuery = useQuery({
    queryKey: ['content', 'item', memberId, contentId] as const,
    queryFn: () => getContentItem(memberId, contentId),
  });
  const [body, setBody] = React.useState('');
  const [showApproval, setShowApproval] = React.useState(false);
  const [scheduledAt, setScheduledAt] = React.useState('');

  React.useEffect(() => {
    if (itemQuery.data) setBody(itemQuery.data.body);
  }, [itemQuery.data]);

  const qc_mutation = useMutation({
    mutationFn: () => runQualityCheck(memberId, contentId),
    onSuccess: (r) => {
      alert(`Quality: ${r.score}, ${r.flags.length} flag(s), coverage ${(r.kb_coverage * 100).toFixed(0)}%`);
    },
  });

  const submitMutation = useMutation({
    mutationFn: () => submitForApproval(memberId, contentId),
    onSuccess: () => {
      setShowApproval(true);
      void qc.invalidateQueries({ queryKey: ['content'] });
    },
  });

  const scheduleMutation = useMutation({
    mutationFn: () => scheduleContent(memberId, contentId, scheduledAt),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ['content'] });
      router.push('/content');
    },
  });

  const deleteMutation = useMutation({
    mutationFn: () => deleteContentItem(memberId, contentId),
    onSuccess: () => router.push('/content'),
  });

  if (itemQuery.isLoading) return <LoadingSkeleton rows={6} />;
  if (itemQuery.error || !itemQuery.data) return <ErrorState onRetry={() => itemQuery.refetch()} />;
  const item = itemQuery.data;

  return (
    <ComplianceGate>
      <div className="mx-auto max-w-3xl space-y-6 p-6">
        <header className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold">{item.title ?? 'Draft editor'}</h1>
            <Badge variant="outline" className="mt-1 capitalize">{item.status}</Badge>
          </div>
          <div className="flex gap-2">
            <Button onClick={() => qc_mutation.mutate()} variant="outline" disabled={qc_mutation.isPending} type="button">
              Quality check
            </Button>
            <Button onClick={() => submitMutation.mutate()} disabled={submitMutation.isPending || item.status !== 'draft'} type="button">
              Submit for approval
            </Button>
          </div>
        </header>

        <Card>
          <CardHeader>
            <CardTitle>Body</CardTitle>
          </CardHeader>
          <CardContent>
            <Textarea value={body} onChange={(e) => setBody(e.target.value)} rows={14} aria-label="Draft body" />
          </CardContent>
        </Card>

        {item.status === 'pending_approval' || item.status === 'approved' ? (
          <Card>
            <CardHeader>
              <CardTitle>Schedule</CardTitle>
            </CardHeader>
            <CardContent className="flex gap-2">
              <input
                type="datetime-local"
                value={scheduledAt}
                onChange={(e) => setScheduledAt(e.target.value)}
                className="rounded-md border bg-background px-3 py-2 text-sm"
                aria-label="Schedule time"
              />
              <Button onClick={() => scheduleMutation.mutate()} disabled={!scheduledAt} type="button">
                Schedule
              </Button>
            </CardContent>
          </Card>
        ) : null}

        <ApprovalDialog
          open={showApproval}
          onOpenChange={setShowApproval}
          approvalId={item.id}
          actionType="publish_post"
          tier={2}
          preview={body}
          targetLabel="LinkedIn post"
          kbRefs={item.kb_refs}
          traceId={item.trace_id}
          idempotencyKey={item.idempotency_key ?? ''}
          onApprove={async () => {
            setShowApproval(false);
            router.push('/content');
          }}
          onReject={async () => {
            setShowApproval(false);
            router.push('/content');
          }}
        />

        <div className="flex justify-end">
          <Button onClick={() => { if (confirm('Delete this draft?')) deleteMutation.mutate(); }} variant="ghost" type="button">
            Delete
          </Button>
        </div>
      </div>
    </ComplianceGate>
  );
}
