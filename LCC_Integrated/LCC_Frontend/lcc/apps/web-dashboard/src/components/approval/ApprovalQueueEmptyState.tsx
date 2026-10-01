'use client';

import * as React from 'react';
import { EmptyState } from '@lcc/ui';
import { CheckCircle2 } from 'lucide-react';

export function ApprovalQueueEmptyState(): React.ReactElement {
  return (
    <EmptyState
      title="No pending approvals"
      description="All actions are either auto-approved or have been cleared. Nice."
      icon={<CheckCircle2 className="h-12 w-12 text-success" />}
      ctaHref="/today"
      ctaLabel="Back to dashboard"
    />
  );
}
