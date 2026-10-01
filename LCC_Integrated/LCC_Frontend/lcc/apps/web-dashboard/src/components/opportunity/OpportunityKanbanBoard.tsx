'use client';

import * as React from 'react';
import { OpportunityKanbanColumn } from './OpportunityKanbanColumn';
import type { Opportunity, OpportunityStage } from '@lcc/api-types';

const COLUMNS: OpportunityStage[] = ['discovered', 'qualified', 'drafting', 'applied', 'interviewing', 'offer', 'won'];

export function OpportunityKanbanBoard({ opps }: { opps: Opportunity[] }): React.ReactElement {
  const grouped = COLUMNS.reduce<Record<OpportunityStage, Opportunity[]>>((acc, s) => {
    acc[s] = [];
    return acc;
  }, {} as Record<OpportunityStage, Opportunity[]>);

  for (const o of opps) {
    const stage = (o.status as OpportunityStage);
    if (grouped[stage]) grouped[stage].push(o);
  }

  return (
    <div className="grid gap-3 md:grid-cols-4 lg:grid-cols-7">
      {COLUMNS.map((c) => (
        <OpportunityKanbanColumn key={c} stage={c} opportunities={grouped[c] ?? []} />
      ))}
    </div>
  );
}
