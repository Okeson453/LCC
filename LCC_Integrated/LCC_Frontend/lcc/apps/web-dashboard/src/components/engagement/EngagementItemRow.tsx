'use client';

import * as React from 'react';
import Link from 'next/link';
import { Badge } from '@lcc/ui';

export interface EngagementItemRowProps {
  id: string;
  actorName: string;
  snippet: string;
  kind: string;
}

export function EngagementItemRow({ id, actorName, snippet, kind }: EngagementItemRowProps): React.ReactElement {
  return (
    <Link href={`/engagement/${id}`} className="flex items-center justify-between rounded-md border p-3 hover:bg-muted/40">
      <div>
        <p className="text-sm font-medium">{actorName}</p>
        <p className="line-clamp-2 text-xs text-muted-foreground">{snippet}</p>
      </div>
      <Badge variant="outline" className="capitalize">{kind}</Badge>
    </Link>
  );
}
