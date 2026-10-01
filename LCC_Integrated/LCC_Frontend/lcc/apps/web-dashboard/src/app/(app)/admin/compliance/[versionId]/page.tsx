'use client';

import { Metadata } from 'next';
import { useQuery } from '@tanstack/react-query';
import { Card, CardContent, CardHeader, CardTitle, Badge, LoadingSkeleton, ErrorState } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';
import { fetchCurrentMember } from '@/lib/api/members';

export const metadata: Metadata = { title: 'Compliance version' };

export default function ComplianceVersionPage({ params }: { params: { versionId: string } }): React.ReactElement {
  return <Version versionId={params.versionId} />;
}

function Version({ versionId }: { versionId: string }): React.ReactElement {
  const q = useQuery({ queryKey: ['members', 'me'] as const, queryFn: fetchCurrentMember });
  if (q.isLoading) return <LoadingSkeleton rows={4} />;
  if (q.error) return <ErrorState onRetry={() => q.refetch()} />;
  return (
    <ComplianceGate>
      <div className="mx-auto max-w-3xl space-y-6 p-6">
        <header>
          <h1 className="text-2xl font-bold">Version {versionId}</h1>
          <Badge variant="secondary">active</Badge>
        </header>
        <Card>
          <CardHeader><CardTitle>Bundle</CardTitle></CardHeader>
          <CardContent>
            <pre className="overflow-auto rounded-md bg-muted p-3 text-xs">
{`version: 3
tier_rules:
  - tier: 1
    requires_approval: false
  - tier: 2
    requires_approval: true
  - tier: 3
    requires_approval: true
    typed_confirmation: false
  - tier: 4
    requires_approval: true
    typed_confirmation: false
  - tier: 5
    requires_approval: true
    typed_confirmation: true
kb_required_tiers: [3, 4, 5]
`}
            </pre>
          </CardContent>
        </Card>
      </div>
    </ComplianceGate>
  );
}
