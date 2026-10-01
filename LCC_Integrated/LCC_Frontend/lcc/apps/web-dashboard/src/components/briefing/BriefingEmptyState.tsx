'use client';

import * as React from 'react';
import { EmptyState } from '@lcc/ui';
import { Calendar } from 'lucide-react';

export function BriefingEmptyState(): React.ReactElement {
  return (
    <EmptyState
      title="No briefing items"
      description="Once we have signals — approvals, opportunities, follow-ups — they appear here."
      icon={<Calendar className="h-12 w-12 text-muted-foreground" />}
    />
  );
}
