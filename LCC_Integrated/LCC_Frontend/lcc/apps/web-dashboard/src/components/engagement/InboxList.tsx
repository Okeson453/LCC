'use client';

import * as React from 'react';
import Link from 'next/link';
import { Card, CardContent, CardHeader, CardTitle, Badge, EmptyState } from '@lcc/ui';

export interface InboxEvent {
  id: string;
  actor: { display_name: string };
  kind: 'comment' | 'mention' | 'message' | 'reaction' | 'connection' | 'tagged';
  snippet: string;
  occurred_at: string;
}

export function InboxList({ items }: { items: InboxEvent[] }): React.ReactElement {
  if (!items.length) return <EmptyState title="Inbox is clear" />;
  return (
    <Card>
      <CardHeader><CardTitle className="text-base">Inbox</CardTitle></CardHeader>
      <CardContent className="space-y-2">
        {items.map((i) => (
          <Link key={i.id} href={`/engagement/${i.id}`} className="flex items-center justify-between rounded-md border p-3 hover:bg-muted/40">
            <div>
              <p className="text-sm font-medium">{i.actor.display_name}</p>
              <p className="line-clamp-2 text-xs text-muted-foreground">{i.snippet}</p>
            </div>
            <Badge variant="outline" className="capitalize">{i.kind}</Badge>
          </Link>
        ))}
      </CardContent>
    </Card>
  );
}
