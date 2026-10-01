'use client';

import * as React from 'react';
import Link from 'next/link';
import { Badge } from '@lcc/ui';
import type { Opportunity } from '@lcc/api-types';

export function OpportunityCard({ opp }: { opp: Opportunity }): React.ReactElement {
  return (
    <Link href={`/opportunities/${opp.id}`} className="block rounded-md border bg-background p-2 hover:bg-muted/40">
      <p className="text-sm font-medium">{opp.title}</p>
      <p className="text-xs text-muted-foreground">{opp.company}</p>
      <div className="mt-1 flex items-center justify-between text-xs">
        <span className="font-mono">φ {opp.fit_score.toFixed(2)}</span>
        <Badge variant="outline" className="text-[10px]">{opp.action_plan}</Badge>
      </div>
    </Link>
  );
}
