'use client';

import { Metadata } from 'next';
import Link from 'next/link';
import { useQuery } from '@tanstack/react-query';
import { Card, CardContent, CardHeader, CardTitle, Badge, Button, LoadingSkeleton, ErrorState } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';
import { fetchCurrentMember } from '@/lib/api/members';
import { listSequences } from '@/lib/api/outreach';

export const metadata: Metadata = { title: 'Outreach' };

export default function OutreachPage(): React.ReactElement {
  return <Outreach />;
}

function Outreach(): React.ReactElement {
  const memberQuery = useQuery({ queryKey: ['members', 'me'] as const, queryFn: fetchCurrentMember });
  if (memberQuery.isLoading) return <LoadingSkeleton rows={6} />;
  if (memberQuery.error || !memberQuery.data) return <ErrorState onRetry={() => memberQuery.refetch()} />;
  return <List memberId={memberQuery.data.id} />;
}

function List({ memberId }: { memberId: string }): React.ReactElement {
  const q = useQuery({ queryKey: ['outreach', 'sequences', memberId] as const, queryFn: () => listSequences(memberId) });
  return (
    <ComplianceGate>
      <div className="space-y-6 p-6">
        <header className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold">Outreach</h1>
            <p className="text-sm text-muted-foreground">Sequences and templates.</p>
          </div>
          <Button asChild>
            <Link href="/outreach/new">New sequence</Link>
          </Button>
        </header>
        <Card>
          <CardHeader><CardTitle className="text-base">Sequences</CardTitle></CardHeader>
          <CardContent className="space-y-2">
            {q.isLoading ? <LoadingSkeleton rows={4} /> :
              !q.data || q.data.length === 0 ? <p className="text-sm text-muted-foreground">No sequences.</p> :
              q.data.map((s) => (
                <Link key={s.id} href={`/outreach/${s.id}`} className="flex items-center justify-between rounded-md border p-3 hover:bg-muted/40">
                  <div>
                    <p className="text-sm font-medium">{s.name}</p>
                    <p className="text-xs text-muted-foreground">{s.step_count} steps · {s.enrolled_count} enrolled</p>
                  </div>
                  <Badge variant="outline" className="capitalize">{s.status}</Badge>
                </Link>
              ))
            }
          </CardContent>
        </Card>
        <Link href="/outreach/templates" className="text-sm text-primary hover:underline">
          Browse templates →
        </Link>
      </div>
    </ComplianceGate>
  );
}
