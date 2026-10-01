'use client';

import * as React from 'react';
import { Card, CardContent, CardHeader, CardTitle, Badge, Button, LoadingSkeleton } from '@lcc/ui';
import { RiskTierBadge, ApprovalDialog } from '@lcc/approval-gate';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { apiFetch } from '@/lib/fetcher';
import { getAccessToken, setAccessToken } from '@/lib/storage';
import type { ApprovalSummary } from '@/lib/types';

export function ApprovalsPanel(): React.ReactElement {
  const qc = useQueryClient();
  const tokenQuery = useQuery({ queryKey: ['extension', 'token'], queryFn: () => getAccessToken() });

  const approvals = useQuery({
    queryKey: ['approvals'],
    queryFn: async () => {
      const r = await apiFetch<{ items: ApprovalSummary[] }>({ method: 'GET', path: '/approvals/queue?status=pending' });
      if (!r.ok) throw new Error(r.error ?? 'fetch_failed');
      return r.data?.items ?? [];
    },
    enabled: Boolean(tokenQuery.data),
  });

  const link = useMutation({
    mutationFn: async (accessToken: string) => { await setAccessToken(accessToken); },
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ['approvals'] });
      void qc.invalidateQueries({ queryKey: ['extension'] });
    },
  });

  return (
    <div className="p-3 space-y-3 text-sm">
      <header className="flex items-center justify-between">
        <h1 className="text-base font-semibold">Approvals</h1>
        <Button onClick={() => approvals.refetch()} variant="ghost" size="sm" type="button">Refresh</Button>
      </header>

      {tokenQuery.isLoading ? <LoadingSkeleton rows={3} /> :
        !tokenQuery.data ? (
          <Card>
            <CardHeader>
              <CardTitle className="text-sm">Sign in</CardTitle>
            </CardHeader>
            <CardContent>
              <button
                onClick={() => { void openWebAuthPage().then((token) => { if (token) link.mutate(token); }); }}
                type="button"
                className="rounded-md bg-primary px-3 py-1 text-xs text-primary-foreground"
              >
                Sign in via web app
              </button>
            </CardContent>
          </Card>
        ) : approvals.isLoading ? <LoadingSkeleton rows={3} /> :
          approvals.error ? <p className="text-xs text-destructive">Failed to load approvals.</p> :
          (approvals.data ?? []).length === 0 ? <p className="text-xs text-muted-foreground">No pending approvals.</p> :
          (
            <ul className="space-y-2">
              {(approvals.data ?? []).map((a) => (
                <ApprovalCard key={a.id} approval={a} />
              ))}
            </ul>
          )
      }
    </div>
  );
}

function ApprovalCard({ approval }: { approval: ApprovalSummary }): React.ReactElement {
  const [open, setOpen] = React.useState(false);
  return (
    <li className="rounded-md border bg-card p-2">
      <div className="flex items-center justify-between gap-2">
        <span className="font-medium">{approval.action_type}</span>
        <div className="flex items-center gap-1">
          <RiskTierBadge tier={approval.tier} dotOnly />
          <Badge variant="outline">{approval.kb_ref_count} KB</Badge>
        </div>
      </div>
      <p className="mt-1 line-clamp-2 text-xs text-muted-foreground">{approval.body_preview}</p>
      <Button onClick={() => setOpen(true)} size="sm" className="mt-2" type="button">Review</Button>

      <ApprovalDialog
        open={open}
        onOpenChange={setOpen}
        approvalId={approval.id}
        actionType={approval.action_type}
        tier={approval.tier}
        preview={approval.body_preview}
        targetLabel={approval.target_label}
        kbRefs={[]}
        traceId={approval.trace_id}
        idempotencyKey={approval.idempotency_key}
        editablePreview={approval.tier >= 3}
        onApprove={async (input) => {
          await apiFetch({ method: 'POST', path: `/approvals/${approval.id}/decide`, body: { decision: 'approve', edited_preview: input.editedPreview } });
          setOpen(false);
        }}
        onReject={async () => {
          await apiFetch({ method: 'POST', path: `/approvals/${approval.id}/decide`, body: { decision: 'reject' } });
          setOpen(false);
        }}
      />
    </li>
  );
}

async function openWebAuthPage(): Promise<string | null> {
  const url = chrome.runtime.getURL('sidepanel/index.html') + '#sign-in';
  await chrome.tabs.create({ url });
  return null;
}
