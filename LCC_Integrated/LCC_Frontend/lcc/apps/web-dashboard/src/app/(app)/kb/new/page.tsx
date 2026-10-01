'use client';

import { Metadata } from 'next';
import { useMutation, useQuery } from '@tanstack/react-query';
import { useRouter } from 'next/navigation';
import { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle, Button, Input, Textarea, LoadingSkeleton, ErrorState, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';
import { fetchCurrentMember } from '@/lib/api/members';
import { createKbRecord } from '@/lib/api/kb';
import type { KbCategory } from '@lcc/api-types';

export const metadata: Metadata = { title: 'New KB record' };

export default function NewKbPage(): React.ReactElement {
  return <NewKb />;
}

function NewKb(): React.ReactElement {
  const memberQuery = useQuery({ queryKey: ['members', 'me'] as const, queryFn: fetchCurrentMember });
  if (memberQuery.isLoading) return <LoadingSkeleton rows={4} />;
  if (memberQuery.error || !memberQuery.data) return <ErrorState onRetry={() => memberQuery.refetch()} />;
  return <Form memberId={memberQuery.data.id} />;
}

function Form({ memberId }: { memberId: string }): React.ReactElement {
  const router = useRouter();
  const [title, setTitle] = useState('');
  const [body, setBody] = useState('');
  const [category, setCategory] = useState<KbCategory>('resume');

  const m = useMutation({
    mutationFn: () => createKbRecord(memberId, { title, body, category }),
    onSuccess: (r) => router.push(`/kb/${r.id}`),
  });

  return (
    <ComplianceGate>
      <div className="mx-auto max-w-3xl space-y-6 p-6">
        <Card>
          <CardHeader><CardTitle>New KB record</CardTitle></CardHeader>
          <CardContent className="space-y-3">
            <Input value={title} onChange={(e) => setTitle(e.target.value)} placeholder="Title" aria-label="Title" />
            <Select value={category} onValueChange={(v) => setCategory(v as KbCategory)}>
              <SelectTrigger><SelectValue /></SelectTrigger>
              <SelectContent>
                <SelectItem value="resume">Resume</SelectItem>
                <SelectItem value="portfolio">Portfolio</SelectItem>
                <SelectItem value="voice_sample">Voice sample</SelectItem>
                <SelectItem value="case_study">Case study</SelectItem>
                <SelectItem value="ideal_customer">ICP</SelectItem>
                <SelectItem value="content_pillar">Content pillar</SelectItem>
                <SelectItem value="goal">Goal</SelectItem>
          </SelectContent>
            </Select>
            <Textarea value={body} onChange={(e) => setBody(e.target.value)} rows={12} placeholder="Body" aria-label="Body" />
            <Button onClick={() => m.mutate()} disabled={m.isPending || !title || !body} type="button">
              {m.isPending ? 'Creating…' : 'Create'}
            </Button>
          </CardContent>
        </Card>
      </div>
    </ComplianceGate>
  );
}
