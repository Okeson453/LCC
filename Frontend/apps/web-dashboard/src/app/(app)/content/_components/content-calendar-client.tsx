'use client';

import * as React from 'react';
import Link from 'next/link';
import { useQuery } from '@tanstack/react-query';
import { Calendar, Plus } from 'lucide-react';
import { Button, Card, CardContent, CardHeader, CardTitle, Badge, EmptyState, LoadingSkeleton, ErrorState } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';
import { fetchCurrentMember } from '@/lib/api/members';
import { listContent } from '@/lib/api/content';

export function ContentCalendar(): React.ReactElement {
  const memberQuery = useQuery({ queryKey: ['members', 'me'] as const, queryFn: fetchCurrentMember });
  if (memberQuery.isLoading) return <LoadingSkeleton rows={6} />;
  if (memberQuery.error || !memberQuery.data) return <ErrorState onRetry={() => memberQuery.refetch()} />;
  return <CalendarView memberId={memberQuery.data.id} />;
}

function CalendarView({ memberId }: { memberId: string }): React.ReactElement {
  const q = useQuery({
    queryKey: ['content', 'list', memberId] as const,
    queryFn: () => listContent(memberId),
  });

  const drafts = (q.data ?? []).filter((c) => c.status === 'draft');
  const scheduled = (q.data ?? []).filter((c) => c.status === 'scheduled');
  const published = (q.data ?? []).filter((c) => c.status === 'published');

  return (
    <ComplianceGate>
      <div className="space-y-6 p-6">
        <header className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold flex items-center gap-2">
              <Calendar className="h-5 w-5" aria-hidden="true" />
              Content calendar
            </h1>
            <p className="text-sm text-muted-foreground">Plan, schedule, and review content.</p>
          </div>
          <Button asChild>
            <Link href="/content/new"><Plus className="h-4 w-4" aria-hidden="true" /> New post</Link>
          </Button>
        </header>

        <div className="grid gap-6 lg:grid-cols-3">
          <Bucket title="Drafts" items={drafts} />
          <Bucket title="Scheduled" items={scheduled} />
          <Bucket title="Published" items={published} />
        </div>
      </div>
    </ComplianceGate>
  );
}

function Bucket({ title, items }: { title: string; items: { id: string; title: string | null; body: string; status: string; scheduled_at: string | null; published_at: string | null }[] }): React.ReactElement {
  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="text-base">{title}</CardTitle>
          <Badge variant="secondary">{items.length}</Badge>
        </div>
      </CardHeader>
      <CardContent className="space-y-2">
        {items.length === 0 ? (
          <EmptyState title="No items" />
        ) : (
          items.map((i) => (
            <Link key={i.id} href={`/content/${i.id}`} className="block rounded-md border p-3 hover:bg-muted/40">
              <p className="text-sm font-medium">{i.title ?? '(untitled)'}</p>
              <p className="mt-1 line-clamp-2 text-xs text-muted-foreground">{i.body.slice(0, 160)}</p>
              <p className="mt-1 text-xs text-muted-foreground">
                {i.scheduled_at ? `Scheduled: ${new Date(i.scheduled_at).toLocaleString()}` : null}
                {i.published_at ? `Published: ${new Date(i.published_at).toLocaleString()}` : null}
              </p>
            </Link>
          ))
        )}
      </CardContent>
    </Card>
  );
}
