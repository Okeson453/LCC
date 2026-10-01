'use client';

import * as React from 'react';
import { Card, CardContent, CardHeader, CardTitle, Button } from '@lcc/ui';

export function QueueDailyRitualCard({ itemCount, day }: { itemCount: number; day: string }): React.ReactElement {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">Daily ritual · {day}</CardTitle>
      </CardHeader>
      <CardContent className="flex items-center justify-between">
        <p className="text-sm">{itemCount} engagements queued</p>
        <Button asChild size="sm" type="button"><a href="/engagement">Open queue</a></Button>
      </CardContent>
    </Card>
  );
}
