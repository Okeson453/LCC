'use client';

import * as React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@lcc/ui';
import type { ProfileEdit } from '@lcc/api-types';

export function ProfileEditList({ edits }: { edits: ProfileEdit[] }): React.ReactElement {
  return (
    <Card>
      <CardHeader><CardTitle className="text-sm">Pending edits ({edits.length})</CardTitle></CardHeader>
      <CardContent className="space-y-2">
        {edits.map((e) => (
          <div key={e.id} className="rounded-md border p-2 text-sm">
            <p><span className="font-mono text-xs">{e.field}</span>: {String(e.proposed_value)}</p>
            <p className="mt-1 text-xs text-muted-foreground">Status: {e.status}</p>
          </div>
        ))}
      </CardContent>
    </Card>
  );
}
