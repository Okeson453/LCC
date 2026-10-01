'use client';

import { Metadata } from 'next';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle, Button, Input, LoadingSkeleton, ErrorState } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';
import { fetchCurrentMember } from '@/lib/api/members';

export const metadata: Metadata = { title: 'Profile settings' };

export default function ProfileSettingsPage(): React.ReactElement {
  return <ProfileSettings />;
}

function ProfileSettings(): React.ReactElement {
  const qc = useQueryClient();
  const memberQuery = useQuery({ queryKey: ['members', 'me'] as const, queryFn: fetchCurrentMember });
  const [displayName, setDisplayName] = useState('');
  const [headline, setHeadline] = useState('');

  useEffect(() => {
    if (memberQuery.data) {
      setDisplayName(memberQuery.data.display_name);
      setHeadline(memberQuery.data.headline);
    }
  }, [memberQuery.data]);

  const save = useMutation({
    mutationFn: async () => ({ ok: true }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['members'] }),
  });

  if (memberQuery.isLoading) return <LoadingSkeleton rows={4} />;
  if (!memberQuery.data) return <ErrorState onRetry={() => memberQuery.refetch()} />;

  return (
    <ComplianceGate>
      <div className="mx-auto max-w-2xl space-y-6 p-6">
        <header>
          <h1 className="text-2xl font-bold">Profile</h1>
        </header>
        <Card>
          <CardHeader><CardTitle>Identity</CardTitle></CardHeader>
          <CardContent className="space-y-3">
            <Input value={displayName} onChange={(e) => setDisplayName(e.target.value)} placeholder="Display name" aria-label="Display name" />
            <Input value={headline} onChange={(e) => setHeadline(e.target.value)} placeholder="Headline" aria-label="Headline" />
            <Button onClick={() => save.mutate()} disabled={save.isPending} type="button">Save</Button>
          </CardContent>
        </Card>
      </div>
    </ComplianceGate>
  );
}
