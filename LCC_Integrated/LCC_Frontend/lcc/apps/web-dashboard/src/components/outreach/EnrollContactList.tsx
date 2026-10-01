'use client';

import * as React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@lcc/ui';

export function EnrollContactList({ enrolled }: { enrolled: { id: string; display_name: string }[] }): React.ReactElement {
  return (
    <Card>
      <CardHeader><CardTitle className="text-sm">Enrolled contacts ({enrolled.length})</CardTitle></CardHeader>
      <CardContent>
        <ul className="space-y-1 text-sm">
          {enrolled.map((c) => <li key={c.id} className="rounded-md border p-2">{c.display_name}</li>)}
        </ul>
      </CardContent>
    </Card>
  );
}
