'use client';

import * as React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@lcc/ui';

export interface AnalyticsKpiTileProps {
  label: string;
  value: string;
  delta?: { value: number; direction: 'up' | 'down' | 'flat' };
  caption?: string;
}

export function AnalyticsKpiTile({ label, value, delta, caption }: AnalyticsKpiTileProps): React.ReactElement {
  const dirColor = delta?.direction === 'up' ? 'text-success' : delta?.direction === 'down' ? 'text-destructive' : 'text-muted-foreground';
  return (
    <Card>
      <CardHeader><CardTitle className="text-xs uppercase tracking-wide">{label}</CardTitle></CardHeader>
      <CardContent>
        <p className="text-2xl font-bold">{value}</p>
        {delta ? (
          <p className={`text-xs ${dirColor}`}>
            {delta.direction === 'up' ? '↑' : delta.direction === 'down' ? '↓' : '→'} {Math.abs(delta.value).toFixed(1)}%
          </p>
        ) : null}
        {caption ? <p className="text-xs text-muted-foreground">{caption}</p> : null}
      </CardContent>
    </Card>
  );
}
