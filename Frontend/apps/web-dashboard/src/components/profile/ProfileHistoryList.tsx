'use client';

import * as React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@lcc/ui';
import type { ProfileHistoryEntry } from '@lcc/api-types';

export function ProfileHistoryList({ history }: { history: ProfileHistoryEntry[] }): React.ReactElement {
  return (
    <Card>
      <CardHeader><CardTitle className="text-sm">History ({history.length})</CardTitle></CardHeader>
      <CardContent className="space-y-2">
        {history.map((h, i) => (
          <div key={i} className="rounded-md border p-2 text-sm">
            <p>{h.field}: {String(h.previous_value)} → {String(h.next_value)}</p>
            <p className="mt-1 text-xs text-muted-foreground">{new Date(h.changed_at).toLocaleString()}</p>
          </div>
        ))}
      </CardContent>
    </Card>
  );
}
