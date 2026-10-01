'use client';

import { Metadata } from 'next';
import { useQuery, useMutation } from '@tanstack/react-query';
import { Card, CardContent, CardHeader, CardTitle, Button, Badge, LoadingSkeleton, ErrorState } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';
import { fetchCurrentMember } from '@/lib/api/members';

export const metadata: Metadata = { title: 'OAuth connections' };

export default function OAuthPage(): React.ReactElement {
  return <Connections />;
}

function Connections(): React.ReactElement {
  const memberQuery = useQuery({ queryKey: ['members', 'me'] as const, queryFn: fetchCurrentMember });
  if (memberQuery.isLoading) return <LoadingSkeleton rows={3} />;
  if (memberQuery.error || !memberQuery.data) return <ErrorState onRetry={() => memberQuery.refetch()} />;
  return <Body memberId={memberQuery.data.id} />;
}

function Body({ memberId }: { memberId: string }): React.ReactElement {
  const revoke = useMutation({ mutationFn: async () => ({ ok: true }) });

  return (
    <ComplianceGate>
      <div className="mx-auto max-w-2xl space-y-6 p-6">
        <header>
          <h1 className="text-2xl font-bold">OAuth connections</h1>
        </header>
        <Card>
          <CardHeader><CardTitle>LinkedIn</CardTitle></CardHeader>
          <CardContent className="flex items-center justify-between">
            <Badge variant="success">Connected</Badge>
            <Button onClick={() => revoke.mutate()} variant="outline" disabled type="button">Revoke (via front-end disabled)</Button>
          </CardContent>
        </Card>
      </div>
    </ComplianceGate>
  );
}
