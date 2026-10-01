'use client';

import { Metadata } from 'next';
import { useMutation, useQuery } from '@tanstack/react-query';
import { Card, CardContent, CardHeader, CardTitle, Button, LoadingSkeleton, ErrorState } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';
import { fetchCurrentMember } from '@/lib/api/members';
import { discoverOpportunities } from '@/lib/api/opportunity';
import { useRouter } from 'next/navigation';

export const metadata: Metadata = { title: 'Run discovery' };

export default function DiscoverPage(): React.ReactElement {
  return <Discover />;
}

function Discover(): React.ReactElement {
  const memberQuery = useQuery({ queryKey: ['members', 'me'] as const, queryFn: fetchCurrentMember });
  const router = useRouter();
  if (memberQuery.isLoading) return <LoadingSkeleton rows={3} />;
  if (memberQuery.error || !memberQuery.data) return <ErrorState onRetry={() => memberQuery.refetch()} />;
  return <Run memberId={memberQuery.data.id} router={router} />;
}

function Run({ memberId, router }: { memberId: string; router: ReturnType<typeof useRouter> }): React.ReactElement {
  const m = useMutation({
    mutationFn: () => discoverOpportunities(memberId),
    onSuccess: () => router.push('/opportunities'),
  });

  return (
    <ComplianceGate>
      <div className="mx-auto max-w-xl space-y-6 p-6">
        <Card>
          <CardHeader>
            <CardTitle>Run discovery</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <p className="text-sm text-muted-foreground">
              We&apos;ll scan LinkedIn, RSS feeds, and your ICP sources for new opportunities. This takes 1–3 minutes.
            </p>
            <Button onClick={() => m.mutate()} disabled={m.isPending} type="button">
              {m.isPending ? 'Running…' : 'Start discovery'}
            </Button>
          </CardContent>
        </Card>
      </div>
    </ComplianceGate>
  );
}
