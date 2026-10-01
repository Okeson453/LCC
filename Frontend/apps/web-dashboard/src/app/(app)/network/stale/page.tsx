'use client';

import { Metadata } from 'next';
import Link from 'next/link';
import { differenceInDays } from 'date-fns';
import { Card, CardContent, CardHeader, CardTitle, Badge, LoadingSkeleton, ErrorState } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';
import { fetchCurrentMember } from '@/lib/api/members';
import { getStaleContacts } from '@/lib/api/network';
import { useQuery } from '@tanstack/react-query';

export const metadata: Metadata = { title: 'Stale contacts' };

export default function StalePage(): React.ReactElement {
  return <StaleClient />;
}

function StaleClient(): React.ReactElement {
  const memberQuery = useQuery({ queryKey: ['members', 'me'] as const, queryFn: fetchCurrentMember });
  if (memberQuery.isLoading) return <LoadingSkeleton rows={6} />;
  if (memberQuery.error || !memberQuery.data) return <ErrorState onRetry={() => memberQuery.refetch()} />;
  return <Stale memberId={memberQuery.data.id} />;
}

function Stale({ memberId }: { memberId: string }): React.ReactElement {
  const q = useQuery({ queryKey: ['network', 'contacts', 'stale', memberId] as const, queryFn: () => getStaleContacts(memberId) });
  const now = new Date();
  return (
    <ComplianceGate>
      <div className="space-y-6 p-6">
        <header>
          <h1 className="text-2xl font-bold">Stale contacts</h1>
          <p className="text-sm text-muted-foreground">Past the staleness threshold.</p>
        </header>
        <Card>
          <CardHeader><CardTitle className="text-base">{q.data?.length ?? 0} contacts</CardTitle></CardHeader>
          <CardContent className="space-y-2">
            {q.isLoading ? <LoadingSkeleton rows={4} /> :
              !q.data || q.data.length === 0 ? <p className="text-sm text-muted-foreground">No stale contacts.</p> :
              q.data.map((c) => {
                const days = c.last_interaction_at ? differenceInDays(now, new Date(c.last_interaction_at)) : null;
                return (
                  <Link key={c.id} href={`/network/${c.id}`} className="flex items-center justify-between rounded-md border p-3 hover:bg-muted/40">
                    <div>
                      <p className="text-sm font-medium">{c.display_name}</p>
                      <p className="text-xs text-muted-foreground">{c.headline}</p>
                    </div>
                    <Badge variant="warning">{days !== null ? `${days}d ago` : 'never'}</Badge>
                  </Link>
                );
              })
            }
          </CardContent>
        </Card>
      </div>
    </ComplianceGate>
  );
}
