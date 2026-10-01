'use client';

/**
 * SidePanelShell — full review surface (larger than the popup).
 * Audit ref: M-33 + A-01 + A-02.
 */
import * as React from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { ApprovalDialog, KbCitationsList, GovernanceTrace, GuardFailurePanel, RiskTierBadge } from '@lcc/approval-gate';
import { Card, CardContent, CardHeader, CardTitle, Badge, Tabs, TabsList, TabsTrigger, TabsContent } from '@lcc/ui';
import { proxyApiRequest } from '../../background/api-client';
import { useExtensionToken } from '../../popup/hooks/use-extension-state';
import { startRestrictedWatcher } from '../../lib/compliance';
import type { ApprovalSummary } from '../../lib/types';

export function SidePanelShell(): React.ReactElement {
  const token = useExtensionToken();
  return (
    <div className="p-3">
      <header className="mb-3 flex items-center justify-between">
        <h1 className="text-base font-semibold">LinkedIn Manager</h1>
        <Badge variant="secondary">{token.hasToken ? 'online' : 'offline'}</Badge>
      </header>
      <Tabs defaultValue="approvals">
        <TabsList>
          <TabsTrigger value="approvals">Approvals</TabsTrigger>
          <TabsTrigger value="compliance">Compliance</TabsTrigger>
          <TabsTrigger value="settings">Settings</TabsTrigger>
        </TabsList>
        <TabsContent value="approvals"><ApprovalsPane /></TabsContent>
        <TabsContent value="compliance"><CompliancePane /></TabsContent>
        <TabsContent value="settings"><SettingsPane /></TabsContent>
      </Tabs>
    </div>
  );
}

function ApprovalsPane(): React.ReactElement {
  const qc = useQueryClient();
  const queue = useQuery({
    queryKey: ['side-queue'],
    queryFn: async () => {
      const res = await proxyApiRequest({ method: 'GET', path: '/api/v1/approvals/queue?status=pending' });
      if (!res.ok) throw new Error(res.error ?? 'fetch_failed');
      return ((res.data as { items?: ApprovalSummary[] })?.items ?? []);
    },
  });
  const decide = useMutation({
    mutationFn: (input: { id: string; decision: 'approve' | 'reject'; edited?: string }) =>
      proxyApiRequest({ method: 'POST', path: `/api/v1/approvals/${input.id}/decide`, body: { decision: input.decision, edited_preview: input.edited } }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['side-queue'] }),
  });
  const [active, setActive] = React.useState<ApprovalSummary | null>(null);

  return (
    <div className="space-y-2">
      {queue.isLoading ? <p className="text-xs">Loading…</p> :
        (queue.data ?? []).length === 0 ? <p className="text-xs text-muted-foreground">Inbox is clear.</p> :
        (queue.data ?? []).map((i) => (
          <Card key={i.id} className="cursor-pointer" onClick={() => setActive(i)}>
            <CardHeader>
              <div className="flex items-center justify-between">
                <CardTitle className="text-xs">{i.action_type}</CardTitle>
                <RiskTierBadge tier={i.tier} dotOnly />
              </div>
            </CardHeader>
            <CardContent>
              <p className="line-clamp-3 text-xs">{i.body_preview}</p>
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
          onApprove={async (inp) => {
            await decide.mutateAsync({ id: active.id, decision: 'approve', edited: inp.editedPreview });
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
    </div>
  );
}

function CompliancePane(): React.ReactElement {
  const [state, setState] = React.useState<{ restricted: boolean; reason: string | null; until: string | null }>({ restricted: false, reason: null, until: null });

  React.useEffect(() => {
    const w = startRestrictedWatcher('me', setState);
    return () => w.stop();
  }, []);

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">Compliance</CardTitle>
      </CardHeader>
      <CardContent className="text-xs">
        {state.restricted ? <span>Restricted · {state.reason ?? 'unknown'}</span> : <span>No restrictions.</span>}
      </CardContent>
    </Card>
  );
}

function SettingsPane(): React.ReactElement {
  return <p className="text-xs text-muted-foreground">Configure via the dashboard → Settings → Account.</p>;
}
