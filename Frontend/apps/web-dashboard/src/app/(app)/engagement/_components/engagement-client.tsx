'use client';

import * as React from 'react';
import Link from 'next/link';
import { useQuery } from '@tanstack/react-query';
import { Card, CardContent, CardHeader, CardTitle, LoadingSkeleton, ErrorState, Tabs, TabsList, TabsTrigger, TabsContent, Badge } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';
import { fetchCurrentMember } from '@/lib/api/members';
import { fetchInbox, fetchQueue } from '@/lib/api/engagement';

export function EngagementClient(): React.ReactElement {
  const memberQuery = useQuery({ queryKey: ['members', 'me'] as const, queryFn: fetchCurrentMember });
  if (memberQuery.isLoading) return <LoadingSkeleton rows={6} />;
  if (memberQuery.error || !memberQuery.data) return <ErrorState onRetry={() => memberQuery.refetch()} />;
  return <EngagementContent memberId={memberQuery.data.id} />;
}

function EngagementContent({ memberId }: { memberId: string }): React.ReactElement {
  const inbox = useQuery({ queryKey: ['engagement', 'inbox', memberId] as const, queryFn: () => fetchInbox(memberId), refetchInterval: 60_000 });
  const queue = useQuery({ queryKey: ['engagement', 'queue', memberId] as const, queryFn: () => fetchQueue(memberId), refetchInterval: 60_000 });

  return (
    <ComplianceGate>
      <div className="space-y-6 p-6">
        <header>
          <h1 className="text-2xl font-bold">Engagement</h1>
          <p className="text-sm text-muted-foreground">Inbox + daily ritual queue.</p>
        </header>
        <Tabs defaultValue="inbox">
          <TabsList>
            <TabsTrigger value="inbox">Inbox</TabsTrigger>
            <TabsTrigger value="queue">Ritual queue</TabsTrigger>
          </TabsList>
          <TabsContent value="inbox">
            <Card>
              <CardHeader><CardTitle>Inbox</CardTitle></CardHeader>
              <CardContent className="space-y-2">
                {inbox.isLoading ? <LoadingSkeleton rows={4} /> :
                  !inbox.data || inbox.data.length === 0 ? <p className="text-sm text-muted-foreground">Inbox is clear.</p> :
                  inbox.data.map((i) => (
                    <Link key={i.id} href={`/engagement/${i.id}`} className="block rounded-md border p-3 hover:bg-muted/40">
                      <div className="flex items-center justify-between">
                        <span className="text-sm font-medium">{i.actor.display_name}</span>
                        <Badge variant="outline" className="capitalize">{i.kind}</Badge>
                      </div>
                      <p className="mt-1 line-clamp-2 text-xs text-muted-foreground">{i.snippet}</p>
                    </Link>
                  ))
                }
              </CardContent>
            </Card>
          </TabsContent>
          <TabsContent value="queue">
            <Card>
              <CardHeader><CardTitle>Ritual queue</CardTitle></CardHeader>
              <CardContent className="space-y-2">
                {queue.isLoading ? <LoadingSkeleton rows={4} /> :
                  !queue.data || queue.data.length === 0 ? <p className="text-sm text-muted-foreground">Nothing queued.</p> :
                  queue.data.map((q) => (
                    <Link key={q.id} href={`/engagement/${q.id}`} className="block rounded-md border p-3 hover:bg-muted/40">
                      <p className="text-sm">{q.body}</p>
                      <p className="mt-1 text-xs text-muted-foreground">{q.actor.display_name}</p>
                    </Link>
                  ))
                }
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      </div>
    </ComplianceGate>
  );
}
