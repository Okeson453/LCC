'use client';

import { Metadata } from 'next';
import { useQuery, useMutation } from '@tanstack/react-query';
import { Card, CardContent, CardHeader, CardTitle, Button, Badge, LoadingSkeleton, ErrorState, Input } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';
import { fetchCurrentMember } from '@/lib/api/members';

export const metadata: Metadata = { title: 'KB & RAG' };

export default function KbRagPage(): React.ReactElement {
  return <KbRag />;
}

function KbRag(): React.ReactElement {
  const memberQuery = useQuery({ queryKey: ['members', 'me'] as const, queryFn: fetchCurrentMember });
  if (memberQuery.isLoading) return <LoadingSkeleton rows={3} />;
  if (memberQuery.error || !memberQuery.data) return <ErrorState onRetry={() => memberQuery.refetch()} />;
  return <Body memberId={memberQuery.data.id} />;
}

function Body({ memberId }: { memberId: string }): React.ReactElement {
  const ingest = useMutation({ mutationFn: async (url: string) => ({ ok: true, url }) });

  return (
    <ComplianceGate>
      <div className="mx-auto max-w-2xl space-y-6 p-6">
        <header>
          <h1 className="text-2xl font-bold">KB & RAG</h1>
        </header>
        <Card>
          <CardHeader>
            <CardTitle>Ingest URL</CardTitle>
          </CardHeader>
          <CardContent className="flex gap-2">
            <Input placeholder="https://…" aria-label="Source URL" />
            <Button onClick={() => ingest.mutate('')} type="button">Ingest</Button>
          </CardContent>
        </Card>
        <Card>
          <CardHeader><CardTitle>Quality gate</CardTitle></CardHeader>
          <CardContent>
            <Badge variant="secondary">Re-ranking disabled</Badge>
          </CardContent>
        </Card>
      </div>
    </ComplianceGate>
  );
}
