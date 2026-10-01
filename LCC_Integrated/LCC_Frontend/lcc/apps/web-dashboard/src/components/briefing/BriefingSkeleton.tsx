'use client';

import * as React from 'react';
import { Skeleton, Card, CardContent, CardHeader } from '@lcc/ui';

export function BriefingSkeleton({ rows = 3 }: { rows?: number }): React.ReactElement {
  return (
    <div className="space-y-3" aria-busy="true">
      {Array.from({ length: rows }).map((_, i) => (
        <Card key={i}>
          <CardHeader><Skeleton className="h-4 w-32" /></CardHeader>
          <CardContent><Skeleton className="h-10 w-full" /></CardContent>
        </Card>
      ))}
    </div>
  );
}
