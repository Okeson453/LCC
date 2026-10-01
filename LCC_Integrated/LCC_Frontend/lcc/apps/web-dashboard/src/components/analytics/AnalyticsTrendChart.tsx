'use client';

import * as React from 'react';
import { AnalyticsChart, type SeriesPoint } from './AnalyticsChart';

export function AnalyticsTrendChart({ title, points }: { title: string; points: SeriesPoint[] }): React.ReactElement {
  return <AnalyticsChart title={title} series={points} />;
}
