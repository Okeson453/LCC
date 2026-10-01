'use client';

import { Metadata } from 'next';
import { useQuery } from '@tanstack/react-query';
import { Card, CardContent, CardHeader, CardTitle, LoadingSkeleton, ErrorState, Badge } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';
import { fetchCurrentMember } from '@/lib/api/members';

export const metadata: Metadata = { title: 'Admin · Members' };

export default function AdminMembersPage(): React.ReactElement {
  return <Members />;
}

function Members(): React.ReactElement {
  const q = useQuery({ queryKey: ['members', 'me'] as const, queryFn: fetchCurrentMember });
  if (q.isLoading) return <LoadingSkeleton rows={4} />;
  if (q.error) return <ErrorState onRetry={() => q.refetch()} />;
  return (
    <ComplianceGate>
      <div className="space-y-6 p-6">
        <header>
          <h1 className="text-2xl font-bold">Members</h1>
        </header>
        <Card>
          <CardHeader><CardTitle>All members</CardTitle></CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">Search and member-level admin actions appear here.</p>
            <div className="mt-3"><Badge variant="secondary">{(q.data?.role)}</Badge></div>
          </CardContent>
        </Card>
      </div>
    </ComplianceGate>
  );
}
