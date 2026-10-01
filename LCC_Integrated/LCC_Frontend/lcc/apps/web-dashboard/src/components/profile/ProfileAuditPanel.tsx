'use client';

import * as React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@lcc/ui';

export function ProfileAuditPanel({ audit }: { audit: unknown }): React.ReactElement {
  return (
    <Card>
      <CardHeader><CardTitle className="text-sm">Audit</CardTitle></CardHeader>
      <CardContent>
        <pre className="overflow-auto rounded-md bg-muted p-3 text-xs">{JSON.stringify(audit, null, 2)}</pre>
      </CardContent>
    </Card>
  );
}
