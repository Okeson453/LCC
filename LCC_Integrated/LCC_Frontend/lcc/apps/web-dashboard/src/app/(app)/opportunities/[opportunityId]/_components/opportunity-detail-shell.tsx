'use client';

import * as React from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Card, CardContent, CardHeader, CardTitle, Badge, Button, LoadingSkeleton, ErrorState, Tabs, TabsList, TabsTrigger, TabsContent } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';
import { fetchCurrentMember } from '@/lib/api/members';
import { getOpportunity, draftProposal } from '@/lib/api/opportunity';

export function OpportunityDetailShell({ opportunityId }: { opportunityId: string }): React.ReactElement {
  const memberQuery = useQuery({ queryKey: ['members', 'me'] as const, queryFn: fetchCurrentMember });
  if (memberQuery.isLoading) return <LoadingSkeleton rows={5} />;
  if (memberQuery.error || !memberQuery.data) return <ErrorState onRetry={() => memberQuery.refetch()} />;
  return <Detail memberId={memberQuery.data.id} opportunityId={opportunityId} />;
}

function Detail({ memberId, opportunityId }: { memberId: string; opportunityId: string }): React.ReactElement {
    const qc = useQueryClient();
const q = useQuery({
    queryKey: ['opportunity', 'item', memberId, opportunityId] as const,
    queryFn: () => getOpportunity(memberId, opportunityId),
  });

  if (q.isLoading) return <LoadingSkeleton rows={5} />;
  if (!q.data) return <ErrorState title="Not found" />;
  const o = q.data;
  const stages = ['discovered', 'qualified', 'drafting', 'applied', 'interviewing', 'offer', 'won'] as const;

  return (
    <ComplianceGate>
      <div className="space-y-6 p-6">
        <header>
          <h1 className="text-2xl font-bold">{o.title}</h1>
          <p className="text-sm text-muted-foreground">
            {o.company} · fit={o.fit_score.toFixed(2)}
          </p>
          <div className="mt-2 flex flex-wrap gap-1">
            {stages.map((s) => (
              <Button
                key={s}
                size="sm"
                variant={o.status === s ? 'default' : 'outline'}
                className="capitalize"
                onClick={() => {
                  qc.setQueryData(['opportunity', 'item', memberId, opportunityId], (prev: any) => (prev ? ({ ...prev, status: s } as any) : prev));
                  console.log('[opportunity] stage optimistically set to', s);
                }}
                type="button"
              >
                {s}
              </Button>
            ))}
          </div>
        </header>

        <Tabs defaultValue="overview">
          <TabsList>
            <TabsTrigger value="overview">Overview</TabsTrigger>
            <TabsTrigger value="evidence">Evidence</TabsTrigger>
            <TabsTrigger value="actions">Actions</TabsTrigger>
          </TabsList>
          <TabsContent value="overview">
            <Card>
              <CardHeader><CardTitle>Why this fits</CardTitle></CardHeader>
              <CardContent>
                <p className="text-sm text-muted-foreground">{o.description ?? 'No description provided.'}</p>
              </CardContent>
            </Card>
          </TabsContent>
                    <TabsContent value="evidence">
            <Card>
              <CardHeader><CardTitle>Signals</CardTitle></CardHeader>
              <CardContent className="space-y-2">
                {(o.hidden_signals ?? []).length > 0 ? (
                  (o.hidden_signals ?? []).map((sig, i) => (
                    <div key={i} className="rounded-md border p-2 text-sm">
                      <p>{sig}</p>
                      <p className="mt-1 text-xs text-muted-foreground">Internal signal captured by scoring</p>
                    </div>
                  ))
                ) : (
                  <p className="text-sm text-muted-foreground">No signals available for this opportunity yet.</p>
                )}
              </CardContent>
            </Card>
          </TabsContent>
                    <TabsContent value="actions">
            <Card>
              <CardHeader><CardTitle>Plan</CardTitle></CardHeader>
              <CardContent className="space-y-3">
                <div className="flex items-center gap-2">
                  <span className="text-sm font-medium">Recommended plan:</span>
                  <Badge variant="outline" className="capitalize">{typeof o.action_plan === 'string' ? o.action_plan : 'Not specified'}</Badge>
                </div>
                <div>
                  <p className="text-sm font-medium mb-1">Fit breakdown</p>
                  <ul className="space-y-1 text-xs text-muted-foreground">
                    {Object.entries(o.fit_breakdown ?? {}).map(([key, val]) => (
                      <li key={key} className="capitalize">{key}: {(val as number).toFixed(0)}%</li>
                    ))}
                  </ul>
                </div>
                <div>
                  <p className="text-sm font-medium mb-1">Requirements</p>
                  {o.requirements && o.requirements.length > 0 ? (
                    <ul className="list-disc pl-5 space-y-1 text-sm text-muted-foreground">
                      {o.requirements.map((req, i) => <li key={i}>{req}</li>)}
                    </ul>
                  ) : (
                    <p className="text-xs text-muted-foreground">No requirements listed.</p>
                  )}
                </div>
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      </div>
    </ComplianceGate>
  );
}
