'use client';

import * as React from 'react';
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@lcc/ui';

export interface AnalyticsRangeTabsProps {
  ranges: { value: '7d' | '30d' | '90d'; label: string; content: React.ReactNode }[];
}

export function AnalyticsRangeTabs({ ranges }: AnalyticsRangeTabsProps): React.ReactElement {
  const first = ranges[0]?.value ?? '7d';
  return (
    <Tabs defaultValue={first}>
      <TabsList>
        {ranges.map((r) => <TabsTrigger key={r.value} value={r.value}>{r.label}</TabsTrigger>)}
      </TabsList>
      {ranges.map((r) => <TabsContent key={r.value} value={r.value}>{r.content}</TabsContent>)}
    </Tabs>
  );
}
