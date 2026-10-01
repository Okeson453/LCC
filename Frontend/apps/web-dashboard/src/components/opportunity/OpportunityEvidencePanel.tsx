'use client';

import * as React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@lcc/ui';
import type { OpportunityEvidence } from '@lcc/api-types';

export function OpportunityEvidencePanel({ evidence }: { evidence: OpportunityEvidence[] }): React.ReactElement {
  return (
    <Card>
      <CardHeader><CardTitle className="text-sm">Signals ({evidence.length})</CardTitle></CardHeader>
      <CardContent className="space-y-2">
        {evidence.map((e, i) => (
          <div key={i} className="rounded-md border p-2 text-sm">
            <p>{e.snippet}</p>
            <p className="mt-1 text-xs text-muted-foreground">
              {new Date(e.observed_at).toLocaleString()} · {e.source}
            </p>
          </div>
        ))}
      </CardContent>
    </Card>
  );
}
