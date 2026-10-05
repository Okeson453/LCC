'use client';

import * as React from 'react';
import { Button, Badge } from '@lcc/ui';
import { RiskTierBadge } from '@lcc/approval-gate';
import type { RiskTier, SequenceStep } from '@lcc/api-types';

export interface SequenceStepRowProps {
  step: SequenceStep;
  /**
   * Risk tier for the step.
   *
   * `SequenceStep` (generated/http/outreach.ts) carries no `tier` field, so
   * this is optional and the badge is only rendered when a caller supplies
   * one. It previously read `step.tier` unconditionally, which did not
   * compile and would have shown a fabricated tier.
   */
  tier?: RiskTier;
  onReview: () => void;
}

export function SequenceStepRow({ step, tier, onReview }: SequenceStepRowProps): React.ReactElement {
  return (
    <div className="rounded-md border p-3">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <span className="text-xs font-mono text-muted-foreground">D+{step.day_offset}</span>
          <Badge variant="outline" className="capitalize">{step.status}</Badge>
          {tier ? <RiskTierBadge tier={tier} dotOnly /> : null}
        </div>
        {step.status === 'pending_approval' || step.status === 'draft' ? (
          <Button size="sm" onClick={onReview} type="button">Review</Button>
        ) : null}
      </div>
      <p className="mt-2 text-sm">{step.body}</p>
      {step.scheduled_at ? (
        <p className="mt-1 text-xs text-muted-foreground">
          Scheduled: {new Date(step.scheduled_at).toLocaleString()}
        </p>
      ) : null}
    </div>
  );
}
