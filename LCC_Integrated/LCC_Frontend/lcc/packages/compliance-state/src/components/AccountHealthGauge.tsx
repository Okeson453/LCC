'use client';

import * as React from 'react';
import { Card, CardContent, CardHeader, CardTitle, Badge } from '@lcc/ui';

export function AccountHealthGauge({ score, threshold, label }: { score: number; threshold: number; label?: string }): React.ReactElement {
  const healthy = score >= threshold;
  return (
    <Card>
      <CardHeader><CardTitle className="text-sm">{label ?? 'Account health'}</CardTitle></CardHeader>
      <CardContent>
        <div
          aria-label={`score ${Math.round(score * 100)} of ${Math.round(threshold * 100)}`}
          className="relative h-20 w-20 rounded-full border-4 border-muted"
        >
          <div className={`absolute inset-1 m-auto h-16 w-16 rounded-full ${healthy ? 'bg-success' : 'bg-warning'}`} />
        </div>
        <p className="mt-2 text-xs text-muted-foreground">
          Score: <Badge variant={healthy ? 'success' : 'warning'}>{Math.round(score * 100)}</Badge> / Threshold: {Math.round(threshold * 100)}
        </p>
      </CardContent>
    </Card>
  );
}
