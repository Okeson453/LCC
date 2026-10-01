'use client';

import { Metadata } from 'next';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Card, CardContent, CardHeader, CardTitle, Badge, Button, LoadingSkeleton, ErrorState, Textarea, Input } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';
import { fetchCurrentMember } from '@/lib/api/members';
import { getKbRecord, updateKbRecord } from '@/lib/api/kb';
import { useState, useEffect } from 'react';

export const metadata: Metadata = { title: 'KB record' };

export default function KbRecordPage({ params }: { params: { recordId: string } }): React.ReactElement {
  return <Record recordId={params.recordId} />;
}

function Record({ recordId }: { recordId: string }): React.ReactElement {
  const memberQuery = useQuery({ queryKey: ['members', 'me'] as const, queryFn: fetchCurrentMember });
  if (memberQuery.isLoading) return <LoadingSkeleton rows={4} />;
  if (memberQuery.error || !memberQuery.data) return <ErrorState onRetry={() => memberQuery.refetch()} />;
  return <Detail memberId={memberQuery.data.id} recordId={recordId} />;
}

function Detail({ memberId, recordId }: { memberId: string; recordId: string }): React.ReactElement {
  const qc = useQueryClient();
  const q = useQuery({ queryKey: ['kb', 'record', memberId, recordId] as const, queryFn: () => getKbRecord(memberId, recordId) });
  const [title, setTitle] = useState('');
  const [body, setBody] = useState('');

  useEffect(() => {
    if (q.data) {
      setTitle(q.data.title);
      setBody(q.data.body);
    }
  }, [q.data]);

  const update = useMutation({
    mutationFn: () => updateKbRecord(memberId, recordId, { title, body }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['kb'] }),
  });

  if (q.isLoading) return <LoadingSkeleton rows={4} />;
  if (!q.data) return <ErrorState title="Not found" />;
  const r = q.data;

  return (
    <ComplianceGate>
      <div className="mx-auto max-w-3xl space-y-6 p-6">
        <header className="flex items-center justify-between">
          <div>
            <Badge variant="outline">{r.category}</Badge>
          </div>
          <Button onClick={() => update.mutate()} disabled={update.isPending} type="button">
            Save
          </Button>
        </header>
        <Card>
          <CardHeader><CardTitle>Title</CardTitle></CardHeader>
          <CardContent>
            <Input value={title} onChange={(e) => setTitle(e.target.value)} aria-label="KB title" />
          </CardContent>
        </Card>
        <Card>
          <CardHeader><CardTitle>Body</CardTitle></CardHeader>
          <CardContent>
            <Textarea value={body} onChange={(e) => setBody(e.target.value)} rows={20} aria-label="KB body" />
          </CardContent>
        </Card>
      </div>
    </ComplianceGate>
  );
}
