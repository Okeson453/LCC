'use client';

import * as React from 'react';
import { SequenceStepRow } from './SequenceStepRow';
import type { RiskTier, SequenceStep } from '@lcc/api-types';

export function SequenceStepList({
  steps,
  tierForStep,
  onReviewStep,
}: {
  steps: SequenceStep[];
  /** Optional per-step risk tier; omitted when the caller has no tier data. */
  tierForStep?: (step: SequenceStep) => RiskTier | undefined;
  onReviewStep: (step: SequenceStep) => void;
}): React.ReactElement {
  return (
    <ol className="space-y-2">
      {steps.map((s) => (
        <li key={s.id}>
          <SequenceStepRow
            step={s}
            tier={tierForStep?.(s)}
            onReview={() => onReviewStep(s)}
          />
        </li>
      ))}
    </ol>
  );
}
