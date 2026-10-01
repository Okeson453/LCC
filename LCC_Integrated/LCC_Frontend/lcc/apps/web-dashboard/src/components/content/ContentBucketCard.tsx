'use client';

import * as React from 'react';
import Link from 'next/link';
import { Card, CardContent, CardHeader, CardTitle, Badge } from '@lcc/ui';

export interface ContentBucketItem {
  id: string;
  title: string | null;
  body: string;
  scheduled_at: string | null;
  published_at: string | null;
}

export interface ContentBucketCardProps {
  title: string;
  items: ContentBucketItem[];
}

export function ContentBucketCard({ title, items }: ContentBucketCardProps): React.ReactElement {
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
          <p className="text-sm text-muted-foreground">No items.</p>
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
