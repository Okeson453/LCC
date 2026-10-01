'use client';

import * as React from 'react';
import { SequenceStepRow } from './SequenceStepRow';
import type { OutreachSequenceStep } from '@lcc/api-types';

export function SequenceStepList({
  steps,
  onReviewStep,
}: {
  steps: OutreachSequenceStep[];
  onReviewStep: (step: OutreachSequenceStep) => void;
}): React.ReactElement {
  return (
    <ol className="space-y-2">
      {steps.map((s) => (
        <li key={s.id}>
          <SequenceStepRow step={s} onReview={() => onReviewStep(s)} />
        </li>
      ))}
    </ol>
  );
}
