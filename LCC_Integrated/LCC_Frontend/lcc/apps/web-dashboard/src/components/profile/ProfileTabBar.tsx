'use client';

import * as React from 'react';
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@lcc/ui';

export function ProfileTabBar({
  tabs,
}: {
  tabs: { value: string; label: string; children: React.ReactNode }[];
}): React.ReactElement {
  const first = tabs[0]?.value ?? 'overview';
  return (
    <Tabs defaultValue={first}>
      <TabsList>
        {tabs.map((t) => <TabsTrigger key={t.value} value={t.value}>{t.label}</TabsTrigger>)}
      </TabsList>
      {tabs.map((t) => <TabsContent key={t.value} value={t.value}>{t.children}</TabsContent>)}
    </Tabs>
  );
}
