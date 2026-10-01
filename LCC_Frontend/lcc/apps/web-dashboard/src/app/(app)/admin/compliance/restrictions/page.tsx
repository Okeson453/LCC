'use client';

import { Metadata } from 'next';
import { useQuery } from '@tanstack/react-query';
import { Card, CardContent, CardHeader, CardTitle, LoadingSkeleton, ErrorState, Badge } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';
import { fetchCurrentMember } from '@/lib/api/members';

export const metadata: Metadata = { title: 'Member restrictions' };

export default function RestrictionsPage(): React.ReactElement {
  return <Restrictions />;
}

function Restrictions(): React.ReactElement {
  const q = useQuery({ queryKey: ['members', 'me'] as const, queryFn: fetchCurrentMember });
  if (q.isLoading) return <LoadingSkeleton rows={4} />;
  if (q.error) return <ErrorState onRetry={() => q.refetch()} />;
  return (
    <ComplianceGate>
      <div className="mx-auto max-w-3xl space-y-6 p-6">
        <header>
          <h1 className="text-2xl font-bold">Member restrictions</h1>
          <p className="text-sm text-muted-foreground">Pause / resume members.</p>
        </header>
        <Card>
          <CardHeader><CardTitle>Active restrictions</CardTitle></CardHeader>
          <CardContent className="space-y-2 text-sm">
            <div className="flex items-center justify-between rounded-md border p-2">
              <span>member-abc-123</span>
              <Badge variant="warning">pause_pending</Badge>
            </div>
          </CardContent>
        </Card>
      </div>
    </ComplianceGate>
  );
}
