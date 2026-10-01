'use client';

import * as React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@lcc/ui';

export function QuotaMeter({ used, cap, label }: { used: number; cap: number; label: string }): React.ReactElement {
  const pct = cap > 0 ? Math.min(100, Math.round((used / cap) * 100)) : 0;
  const near = pct >= 80;
  return (
    <Card>
      <CardHeader><CardTitle className="text-sm">{label}</CardTitle></CardHeader>
      <CardContent className="space-y-1">
        <div className="flex items-center justify-between text-xs">
          <span>Used: {used}</span>
          <span>Cap: {cap}</span>
        </div>
        <div className="h-2 w-full overflow-hidden rounded-full bg-muted">
          <div className={`h-full ${near ? 'bg-warning' : 'bg-primary'}`} style={{ width: `${pct}%` }} />
        </div>
      </CardContent>
    </Card>
  );
}
