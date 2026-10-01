'use client';

import { Metadata } from 'next';
import { useQuery } from '@tanstack/react-query';
import { Card, CardContent, CardHeader, CardTitle, Badge, LoadingSkeleton, ErrorState } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';
import { fetchCurrentMember } from '@/lib/api/members';
import { listSequenceTemplates } from '@/lib/api/outreach';

export const metadata: Metadata = { title: 'Templates' };

export default function TemplatesPage(): React.ReactElement {
  return <Templates />;
}

function Templates(): React.ReactElement {
  const memberQuery = useQuery({ queryKey: ['members', 'me'] as const, queryFn: fetchCurrentMember });
  if (memberQuery.isLoading) return <LoadingSkeleton rows={4} />;
  if (memberQuery.error || !memberQuery.data) return <ErrorState onRetry={() => memberQuery.refetch()} />;
  return <List memberId={memberQuery.data.id} />;
}

function List({ memberId }: { memberId: string }): React.ReactElement {
  const q = useQuery({ queryKey: ['outreach', 'templates', memberId] as const, queryFn: () => listSequenceTemplates(memberId) });
  return (
    <ComplianceGate>
      <div className="space-y-6 p-6">
        <header>
          <h1 className="text-2xl font-bold">Templates</h1>
          <p className="text-sm text-muted-foreground">Persona-keyed sequence templates.</p>
        </header>
        <Card>
          <CardHeader><CardTitle className="text-base">{q.data?.length ?? 0} templates</CardTitle></CardHeader>
          <CardContent className="space-y-2">
            {q.isLoading ? <LoadingSkeleton rows={3} /> :
              q.data?.map((t) => (
                <div key={t.id} className="rounded-md border p-3">
                  <div className="flex items-center justify-between">
                    <p className="text-sm font-medium">{t.name}</p>
                    <Badge variant="outline">{t.persona}</Badge>
                  </div>
                  <p className="mt-1 text-xs text-muted-foreground">{t.body_preview}</p>
                </div>
              ))
            }
          </CardContent>
        </Card>
      </div>
    </ComplianceGate>
  );
}
