'use client';

import * as React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@lcc/ui';

export interface SeriesPoint { x: string; y: number }
export interface AnalyticsChartProps {
  title: string;
  series: SeriesPoint[];
  caption?: string;
}

export function AnalyticsChart({ title, series, caption }: AnalyticsChartProps): React.ReactElement {
  const max = Math.max(1, ...series.map((s) => s.y));
  return (
    <Card>
      <CardHeader><CardTitle className="text-sm">{title}</CardTitle></CardHeader>
      <CardContent>
        <svg viewBox="0 0 320 120" role="img" aria-label={title} className="h-32 w-full">
          <polyline
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            points={series
              .map((s, i) => {
                const x = (i / Math.max(1, series.length - 1)) * 320;
                const y = 120 - (s.y / max) * 120;
                return `${x},${y}`;
              })
              .join(' ')}
          />
          {series.map((s, i) => (
            <circle
              key={i}
              cx={(i / Math.max(1, series.length - 1)) * 320}
              cy={120 - (s.y / max) * 120}
              r={2}
              fill="currentColor"
            />
          ))}
        </svg>
        {caption ? <p className="mt-2 text-xs text-muted-foreground">{caption}</p> : null}
      </CardContent>
    </Card>
  );
}
