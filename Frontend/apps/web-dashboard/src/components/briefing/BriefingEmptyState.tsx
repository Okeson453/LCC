'use client';

import * as React from 'react';
import { EmptyState } from '@lcc/ui';
import { Calendar } from 'lucide-react';

export interface BriefingEmptyStateProps {
  /** Overrides the default copy. Each section passes its own wording. */
  message?: string;
}

export function BriefingEmptyState({ message }: BriefingEmptyStateProps = {}): React.ReactElement {
  return (
    <EmptyState
      title="No briefing items"
      description={message ?? 'Once we have signals — approvals, opportunities, follow-ups — they appear here.'}
      icon={<Calendar className="h-12 w-12 text-muted-foreground" />}
    />
  );
}
