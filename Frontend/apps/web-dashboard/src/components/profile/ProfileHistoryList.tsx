'use client';

import * as React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@lcc/ui';
import type { ProfileStrengthPoint } from '@lcc/api-types';

export function ProfileHistoryList({ history }: { history: ProfileStrengthPoint[] }): React.ReactElement {
  return (
    <Card>
      <CardHeader><CardTitle className="text-sm">History ({history.length})</CardTitle></CardHeader>
      <CardContent className="space-y-2">
        {history.map((h, i) => (
          <div key={i} className="rounded-md border p-2 text-sm">
            <p>Strength: {h.strength}{h.delta ? ` (${h.delta > 0 ? '+' : ''}${h.delta})` : ''}</p>
            <p className="mt-1 text-xs text-muted-foreground">{new Date(h.captured_at).toLocaleString()}</p>
          </div>
        ))}
      </CardContent>
    </Card>
  );
}
