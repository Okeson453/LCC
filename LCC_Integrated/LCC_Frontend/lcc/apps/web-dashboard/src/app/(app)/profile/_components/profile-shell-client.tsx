'use client';

import * as React from 'react';
import Link from 'next/link';
import { useQuery } from '@tanstack/react-query';
import { Card, CardContent, CardHeader, CardTitle, LoadingSkeleton, ErrorState, Tabs, TabsList, TabsTrigger, TabsContent } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';
import { fetchCurrentMember } from '@/lib/api/members';
import type { ProfileView } from '@/lib/api/profile';
import { getProfile, getProfileAudit, getProfileEdits, getProfileHistory } from '@/lib/api/profile';
import type { ProfileSnapshot } from '@lcc/api-types';

export function ProfileShellClient(): React.ReactElement {
  const memberQuery = useQuery({ queryKey: ['members', 'me'] as const, queryFn: fetchCurrentMember });
  if (memberQuery.isLoading) return <LoadingSkeleton rows={6} />;
  if (memberQuery.error || !memberQuery.data) return <ErrorState onRetry={() => memberQuery.refetch()} />;
  return <Body memberId={memberQuery.data.id} />;
}

function Body({ memberId }: { memberId: string }): React.ReactElement {
  const profile = useQuery({ queryKey: ['profile', 'current', memberId] as const, queryFn: () => getProfile(memberId) });
  const audit = useQuery({ queryKey: ['profile', 'audit', memberId] as const, queryFn: () => getProfileAudit(memberId) });
  const edits = useQuery({ queryKey: ['profile', 'edits', memberId] as const, queryFn: () => getProfileEdits(memberId) });
  const history = useQuery({ queryKey: ['profile', 'history', memberId] as const, queryFn: () => getProfileHistory(memberId) });

  if (profile.isLoading) return <LoadingSkeleton rows={5} />;
  if (!profile.data) return <ErrorState title="Profile not found" />;
  const p = profile.data as ProfileView;

  return (
    <ComplianceGate>
      <div className="space-y-6 p-6">
        <header className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold">{p.display_name}</h1>
            <p className="text-sm text-muted-foreground">{p.headline}</p>
          </div>
          <div className="flex gap-2 text-sm">
            <Link href="/profile/audit" className="text-primary hover:underline">Audit</Link>
            <Link href="/profile/edits" className="text-primary hover:underline">Edits</Link>
            <Link href="/profile/history" className="text-primary hover:underline">History</Link>
          </div>
        </header>

        <Tabs defaultValue="overview">
          <TabsList>
            <TabsTrigger value="overview">Overview</TabsTrigger>
            <TabsTrigger value="audit">Audit</TabsTrigger>
            <TabsTrigger value="edits">Edits</TabsTrigger>
            <TabsTrigger value="history">History</TabsTrigger>
          </TabsList>
          <TabsContent value="overview">
            <Card>
              <CardHeader><CardTitle>About</CardTitle></CardHeader>
              <CardContent><p className="whitespace-pre-line text-sm text-muted-foreground">{p.about}</p></CardContent>
            </Card>
            <Card className="mt-4">
              <CardHeader><CardTitle>Experience</CardTitle></CardHeader>
              <CardContent className="space-y-2">
                {p.experience.map((e, i) => (
                  <div key={i} className="rounded-md border p-3">
                    <p className="text-sm font-medium">{e.title} — {e.company}</p>
                    <p className="text-xs text-muted-foreground">{e.starts_at} → {e.ends_at ?? 'present'} · {e.location}</p>
                  </div>
                ))}
              </CardContent>
            </Card>
          </TabsContent>
          <TabsContent value="audit">
            <Card>
              <CardHeader><CardTitle>Audit</CardTitle></CardHeader>
              <CardContent>
                <pre className="overflow-auto rounded-md bg-muted p-3 text-xs">{JSON.stringify(audit.data ?? {}, null, 2)}</pre>
              </CardContent>
            </Card>
          </TabsContent>
          <TabsContent value="edits">
            <Card>
              <CardHeader><CardTitle>Pending edits</CardTitle></CardHeader>
              <CardContent className="space-y-2">
                {(edits.data ?? []).map((e) => (
                  <div key={e.id} className="rounded-md border p-2 text-sm">
                    <p><span className="font-mono text-xs">{e.field}</span>: {String(e.proposed_value)}</p>
                    <p className="mt-1 text-xs text-muted-foreground">Status: {e.status}</p>
                  </div>
                ))}
              </CardContent>
            </Card>
          </TabsContent>
          <TabsContent value="history">
            <Card>
              <CardHeader><CardTitle>History</CardTitle></CardHeader>
              <CardContent className="space-y-2">
                {(history.data ?? []).map((h, i) => (
                  <div key={i} className="rounded-md border p-2 text-sm">
                    <p>{h.field}: {String(h.previous_value)} → {String(h.next_value)}</p>
                    <p className="mt-1 text-xs text-muted-foreground">{new Date(h.changed_at).toLocaleString()}</p>
                  </div>
                ))}
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      </div>
    </ComplianceGate>
  );
}
