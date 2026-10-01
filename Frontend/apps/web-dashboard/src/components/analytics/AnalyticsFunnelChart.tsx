'use client';

import * as React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@lcc/ui';

export interface FunnelStage { name: string; count: number }

export function AnalyticsFunnelChart({ title, stages }: { title: string; stages: FunnelStage[] }): React.ReactElement {
  const max = Math.max(1, ...stages.map((s) => s.count));
  return (
    <Card>
      <CardHeader><CardTitle className="text-sm">{title}</CardTitle></CardHeader>
      <CardContent className="space-y-2">
        {stages.map((s) => (
          <div key={s.name} className="space-y-1">
            <div className="flex items-center justify-between text-xs">
              <span>{s.name}</span>
              <span>{s.count}</span>
            </div>
            <div className="h-2 w-full overflow-hidden rounded-full bg-muted">
              <div className="h-full bg-primary" style={{ width: `${(s.count / max) * 100}%` }} />
            </div>
          </div>
        ))}
      </CardContent>
    </Card>
  );
}
