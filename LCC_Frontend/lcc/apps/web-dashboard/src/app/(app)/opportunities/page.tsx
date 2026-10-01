'use client';

import { Metadata } from 'next';
import Link from 'next/link';
import { useQuery, useMutation } from '@tanstack/react-query';
import { Card, CardContent, CardHeader, CardTitle, Button, Badge, LoadingSkeleton, ErrorState } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';
import { fetchCurrentMember } from '@/lib/api/members';
import { listOpportunities, discoverOpportunities } from '@/lib/api/opportunity';

export const metadata: Metadata = { title: 'Opportunities' };

export default function OpportunitiesPage(): React.ReactElement {
  return <Opps />;
}

function Opps(): React.ReactElement {
  const memberQuery = useQuery({ queryKey: ['members', 'me'] as const, queryFn: fetchCurrentMember });
  if (memberQuery.isLoading) return <LoadingSkeleton rows={6} />;
  if (memberQuery.error || !memberQuery.data) return <ErrorState onRetry={() => memberQuery.refetch()} />;
  return <Board memberId={memberQuery.data.id} />;
}

function Board({ memberId }: { memberId: string }): React.ReactElement {
  const q = useQuery({ queryKey: ['opportunity', 'list', memberId] as const, queryFn: () => listOpportunities(memberId) });
  const discover = useMutation({
    mutationFn: () => discoverOpportunities(memberId),
    onSuccess: () => q.refetch(),
  });

  const grouped = (q.data ?? []).reduce<Record<string, typeof q.data>>((acc, o) => {
    acc[o.status] = acc[o.status] ?? [];
    acc[o.status]!.push(o);
    return acc;
  }, {});

  const cols = ['discovered', 'qualified', 'drafting', 'applied', 'interviewing', 'offer', 'won'];

  return (
    <ComplianceGate>
      <div className="space-y-6 p-6">
        <header className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold">Opportunities</h1>
            <p className="text-sm text-muted-foreground">Pipeline board.</p>
          </div>
          <Button onClick={() => discover.mutate()} disabled={discover.isPending} type="button">
            {discover.isPending ? 'Discovering…' : 'Run discovery'}
          </Button>
        </header>

        <div className="grid gap-4 overflow-x-auto md:grid-cols-4 lg:grid-cols-7">
          {cols.map((c) => (
            <Card key={c}>
              <CardHeader>
                <div className="flex items-center justify-between">
                  <CardTitle className="text-xs uppercase tracking-wide">{c}</CardTitle>
                  <Badge variant="secondary">{grouped[c]?.length ?? 0}</Badge>
                </div>
              </CardHeader>
              <CardContent className="space-y-2">
                {(grouped[c] ?? []).map((o) => (
                  <Link key={o.id} href={`/opportunities/${o.id}`} className="block rounded-md border bg-background p-2 hover:bg-muted/40">
                    <p className="text-sm font-medium">{o.title}</p>
                    <p className="text-xs text-muted-foreground">{o.company}</p>
                    <div className="mt-1 flex items-center justify-between text-xs">
                      <span className="font-mono">φ {o.fit_score.toFixed(2)}</span>
                      <Badge variant="outline" className="text-[10px]">{o.action_plan}</Badge>
                    </div>
                  </Link>
                ))}
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    </ComplianceGate>
  );
}
