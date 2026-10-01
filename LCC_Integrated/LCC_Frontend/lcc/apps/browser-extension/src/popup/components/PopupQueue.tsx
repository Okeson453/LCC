'use client';

/**
 * PopupQueue — Tier 2+ decision queue (focus on next item to review).
 * Audit ref: M-31 + A-01 + A-02.
 */
import * as React from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { RiskTierBadge, ApprovalDialog, KbCitationsList, GovernanceTrace, GuardFailurePanel } from '@lcc/approval-gate';
import { Card, CardContent, CardHeader, CardTitle, Badge, Button } from '@lcc/ui';
import { proxyApiRequest } from '../../background/api-client';
import type { ApprovalSummary } from '../../lib/types';

export function PopupQueue(): React.ReactElement {
  const qc = useQueryClient();
  const queue = useQuery({
    queryKey: ['popup-queue'],
    queryFn: async () => {
      const res = await proxyApiRequest({ method: 'GET', path: '/api/v1/approvals/queue?status=pending' });
      if (!res.ok) throw new Error(res.error ?? 'fetch_failed');
      return ((res.data as { items?: ApprovalSummary[] })?.items ?? []);
    },
  });

  const decide = useMutation({
    mutationFn: (input: { id: string; decision: 'approve' | 'reject'; edited_preview?: string }) =>
      proxyApiRequest({
        method: 'POST',
        path: `/api/v1/approvals/${input.id}/decide`,
        body: { decision: input.decision, edited_preview: input.edited_preview },
      }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['popup-queue'] }),
  });

  const items = queue.data ?? [];
  const [active, setActive] = React.useState<ApprovalSummary | null>(null);

  return (
    <div className="space-y-3 p-3">
      <header className="flex items-center justify-between">
        <h1 className="text-sm font-semibold">Queue</h1>
        <Badge variant="outline">{items.length}</Badge>
      </header>
      {items.length === 0 ? <p className="text-xs text-muted-foreground">No pending approvals.</p> :
        items.map((i) => (
          <Card key={i.id} className="cursor-pointer" onClick={() => setActive(i)}>
            <CardHeader>
              <div className="flex items-center justify-between">
                <CardTitle className="text-xs">{i.action_type}</CardTitle>
                <RiskTierBadge tier={i.tier} dotOnly />
              </div>
            </CardHeader>
            <CardContent>
              <p className="line-clamp-2 text-xs text-muted-foreground">{i.body_preview}</p>
            </CardContent>
          </Card>
        ))
      }

      {active ? (
        <ApprovalDialog
          open
          onOpenChange={(o) => !o && setActive(null)}
          approvalId={active.id}
          actionType={active.action_type}
          tier={active.tier}
          preview={active.body_preview}
          targetLabel="—"
          kbRefs={[]}
          traceId={active.trace_id}
          idempotencyKey={active.idempotency_key}
          editablePreview={active.tier >= 3}
          isSubmitting={decide.isPending}
          onApprove={async (input) => {
            await decide.mutateAsync({ id: active.id, decision: 'approve', edited_preview: input.editedPreview });
            setActive(null);
          }}
          onReject={async () => {
            await decide.mutateAsync({ id: active.id, decision: 'reject' });
            setActive(null);
          }}
          kbCitationsSlot={(refs) => <KbCitationsList citations={refs} />}
          governanceTraceSlot={() => <GovernanceTrace traceId={active.trace_id} idempotencyKey={active.idempotency_key} />}
          guardFailureSlot={(g) => <GuardFailurePanel guards={g} />}
        />
      ) : null}

      <Button className="w-full" onClick={() => chrome.runtime.openOptionsPage?.()}>Open full dashboard</Button>
    </div>
  );
}
