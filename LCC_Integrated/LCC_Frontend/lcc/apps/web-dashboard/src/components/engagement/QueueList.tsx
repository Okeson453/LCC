'use client';

import * as React from 'react';
import Link from 'next/link';
import { Card, CardContent, CardHeader, CardTitle, EmptyState } from '@lcc/ui';

export interface QueueItem {
  id: string;
  body: string;
  actor: { display_name: string };
  scheduled_for: string | null;
}

export function QueueList({ items }: { items: QueueItem[] }): React.ReactElement {
  if (!items.length) return <EmptyState title="Nothing queued" />;
  return (
    <Card>
      <CardHeader><CardTitle className="text-base">Ritual queue</CardTitle></CardHeader>
      <CardContent className="space-y-2">
        {items.map((q) => (
          <Link key={q.id} href={`/engagement/${q.id}`} className="block rounded-md border p-3 hover:bg-muted/40">
            <p className="text-sm">{q.body}</p>
            <p className="mt-1 text-xs text-muted-foreground">{q.actor.display_name}</p>
            {q.scheduled_for ? <p className="text-xs text-muted-foreground">For: {new Date(q.scheduled_for).toLocaleString()}</p> : null}
          </Link>
        ))}
      </CardContent>
    </Card>
  );
}
