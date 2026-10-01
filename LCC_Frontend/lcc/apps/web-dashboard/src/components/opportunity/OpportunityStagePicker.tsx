'use client';

import * as React from 'react';
import { Button } from '@lcc/ui';
import type { OpportunityStage } from '@lcc/api-types';

const STAGES: OpportunityStage[] = ['discovered', 'qualified', 'drafting', 'applied', 'interviewing', 'offer', 'won'];

export function OpportunityStagePicker({ value, onChange }: { value: OpportunityStage; onChange: (s: OpportunityStage) => void | Promise<void> }): React.ReactElement {
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
