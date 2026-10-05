'use client';

import * as React from 'react';
import { Button } from '@lcc/ui';
import type { OpportunityStatus } from '@lcc/api-types';

// Same set the Kanban board renders, so a stage picked here always has a column.
const STAGES: OpportunityStatus[] = ['discovered', 'qualified', 'drafting', 'applied', 'interviewing', 'offer', 'won'];

export function OpportunityStagePicker({ value, onChange }: { value: OpportunityStatus; onChange: (s: OpportunityStatus) => void | Promise<void> }): React.ReactElement {
  return (
    <div className="flex flex-wrap gap-1">
      {STAGES.map((s) => (
        <Button
          key={s}
          size="sm"
          variant={value === s ? 'default' : 'outline'}
          className="capitalize"
          onClick={() => onChange(s)}
          type="button"
        >
          {s}
        </Button>
      ))}
    </div>
  );
}
