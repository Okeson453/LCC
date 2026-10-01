'use client';

import * as React from 'react';
import { Button, Badge } from '@lcc/ui';
import { RiskTierBadge } from '@lcc/approval-gate';
import type { OutreachSequenceStep } from '@lcc/api-types';

export function SequenceStepRow({ step, onReview }: { step: OutreachSequenceStep; onReview: () => void }): React.ReactElement {
  return (
    <div className="rounded-md border p-3">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <span className="text-xs font-mono text-muted-foreground">D+{step.day_offset}</span>
          <Badge variant="outline" className="capitalize">{step.status}</Badge>
          <RiskTierBadge tier={step.tier} dotOnly />
        </div>
        {(step.status === 'pending_approval' || step.status === 'draft') ? (
          <Button size="sm" onClick={onReview} type="button">Review</Button>
        ) : null}
      </div>
      <p className="mt-2 text-sm">{step.body}</p>
      {step.scheduled_at ? <p className="mt-1 text-xs text-muted-foreground">Scheduled: {new Date(step.scheduled_at).toLocaleString()}</p> : null}
    </div>
  );
}
