'use client';

import * as React from 'react';
import { OpportunityKanbanColumn } from './OpportunityKanbanColumn';
import type { Opportunity, OpportunityStatus } from '@lcc/api-types';

/** The subset of `OpportunityStatus` rendered as Kanban columns. */
export type OpportunityBoardColumn = Extract<
  OpportunityStatus,
  'discovered' | 'qualified' | 'drafting' | 'applied' | 'interviewing' | 'offer' | 'won'
>;

const COLUMNS: OpportunityBoardColumn[] = ['discovered', 'qualified', 'drafting', 'applied', 'interviewing', 'offer', 'won'];

export function OpportunityKanbanBoard({ opps }: { opps: Opportunity[] }): React.ReactElement {
  const grouped = COLUMNS.reduce<Record<OpportunityBoardColumn, Opportunity[]>>((acc, s) => {
    acc[s] = [];
    return acc;
  }, {} as Record<OpportunityBoardColumn, Opportunity[]>);

  for (const o of opps) {
    const stage = o.status as OpportunityBoardColumn;
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
