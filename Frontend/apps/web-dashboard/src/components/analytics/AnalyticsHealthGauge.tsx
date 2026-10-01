'use client';

import * as React from 'react';
import { Card, CardContent, CardHeader, CardTitle, Badge } from '@lcc/ui';

export interface AnalyticsHealthGaugeProps {
  label: string;
  score: number;
  threshold: number;
}

export function AnalyticsHealthGauge({ label, score, threshold }: AnalyticsHealthGaugeProps): React.ReactElement {
  const healthy = score >= threshold;
  const tone = healthy ? 'success' : 'warning';
  return (
    <Card>
      <CardHeader><CardTitle className="text-sm">{label}</CardTitle></CardHeader>
      <CardContent className="flex items-center gap-3">
        <div
          aria-label={`${label} ${Math.round(score * 100)} of threshold ${Math.round(threshold * 100)}`}
          className="relative h-16 w-16 rounded-full border-4 border-muted"
        >
          <div
            className={`absolute inset-0 m-auto h-12 w-12 rounded-full bg-${tone}`}
            style={{ transform: `scale(${Math.max(0.2, score)})` }}
          />
        </div>
        <div>
          <Badge variant={tone}>{Math.round(score * 100)} / 100</Badge>
          <p className="text-xs text-muted-foreground">Threshold {Math.round(threshold * 100)}</p>
        </div>
      </CardContent>
    </Card>
  );
}
