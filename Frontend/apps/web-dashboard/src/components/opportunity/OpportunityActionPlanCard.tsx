'use client';

import * as React from 'react';
import { Card, CardContent, CardHeader, CardTitle, Badge } from '@lcc/ui';
import type { ActionItem } from '@lcc/api-types';

export function OpportunityActionPlanCard({ plan, actionItems }: { plan: string; actionItems: ActionItem[] }): React.ReactElement {
  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm">Plan</CardTitle>
          <Badge variant="outline">{plan}</Badge>
        </div>
      </CardHeader>
      <CardContent>
        <ul className="space-y-2">
          {actionItems.map((a, i) => (
            <li key={i} className="rounded-md border p-2 text-sm">
              <p>{a.title}</p>
              {a.due_at ? <p className="mt-1 text-xs text-muted-foreground">Due: {new Date(a.due_at).toLocaleString()}</p> : null}
            </li>
          ))}
        </ul>
      </CardContent>
    </Card>
  );
}
