'use client';

import { Metadata } from 'next';
import { useQuery, useMutation } from '@tanstack/react-query';
import { useRouter } from 'next/navigation';
import { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle, Button, Input, Textarea, LoadingSkeleton, ErrorState } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';
import { fetchCurrentMember } from '@/lib/api/members';
import { createSequence, listSequenceTemplates } from '@/lib/api/outreach';

export const metadata: Metadata = { title: 'New sequence' };

export default function NewSequencePage(): React.ReactElement {
  return <NewSequence />;
}

function NewSequence(): React.ReactElement {
  const memberQuery = useQuery({ queryKey: ['members', 'me'] as const, queryFn: fetchCurrentMember });
  if (memberQuery.isLoading) return <LoadingSkeleton rows={4} />;
  if (memberQuery.error || !memberQuery.data) return <ErrorState onRetry={() => memberQuery.refetch()} />;
  return <Form memberId={memberQuery.data.id} />;
}

function Form({ memberId }: { memberId: string }): React.ReactElement {
  const router = useRouter();
  const templates = useQuery({ queryKey: ['outreach', 'templates', memberId] as const, queryFn: () => listSequenceTemplates(memberId) });
  const [name, setName] = useState('');
  const [templateId, setTemplateId] = useState<string>('');

  const mutation = useMutation({
    mutationFn: () => createSequence(memberId, { name, contact_ids: [], template_id: templateId }),
    onSuccess: (s) => router.push(`/outreach/${s.id}`),
  });

  return (
    <ComplianceGate>
      <div className="mx-auto max-w-2xl space-y-6 p-6">
        <header>
          <h1 className="text-2xl font-bold">New sequence</h1>
        </header>
        <Card>
          <CardHeader><CardTitle>Details</CardTitle></CardHeader>
          <CardContent className="space-y-3">
            <Input value={name} onChange={(e) => setName(e.target.value)} placeholder="Sequence name" aria-label="Sequence name" />
            <select
              value={templateId}
              onChange={(e) => setTemplateId(e.target.value)}
              className="w-full rounded-md border bg-background px-3 py-2 text-sm"
              aria-label="Template"
            >
              <option value="">Select template…</option>
              {templates.data?.map((t) => <option key={t.id} value={t.id}>{t.name}</option>)}
            </select>
            <Button onClick={() => mutation.mutate()} disabled={!name || !templateId || mutation.isPending} type="button">
              {mutation.isPending ? 'Creating…' : 'Create sequence'}
            </Button>
          </CardContent>
        </Card>
      </div>
    </ComplianceGate>
  );
}
