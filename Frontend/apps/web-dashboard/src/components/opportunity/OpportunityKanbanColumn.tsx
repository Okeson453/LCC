'use client';

import * as React from 'react';
import { Card, CardContent, CardHeader, CardTitle, Badge } from '@lcc/ui';
import { OpportunityCard } from './OpportunityCard';
import type { Opportunity, OpportunityStage } from '@lcc/api-types';

export function OpportunityKanbanColumn({ stage, opportunities }: { stage: OpportunityStage; opportunities: Opportunity[] }): React.ReactElement {
  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="text-xs uppercase tracking-wide">{stage}</CardTitle>
          <Badge variant="secondary">{opportunities.length}</Badge>
        </div>
      </CardHeader>
      <CardContent className="space-y-2">
        {opportunities.map((o) => <OpportunityCard key={o.id} opp={o} />)}
      </CardContent>
    </Card>
  );
}
