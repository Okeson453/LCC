'use client';

import * as React from 'react';
import { cn } from '@lcc/ui';

export interface BriefingRootProps {
  children: React.ReactNode;
  className?: string;
  id?: string;
}

/**
 * BriefingRoot — orchestrator that lays out the briefing sections in
 * a responsive grid. Each child section is responsible for its own
 * loading / empty state.
 */
export function BriefingRoot({ children, className, id }: BriefingRootProps): React.ReactElement {
  return (
    <div id={id} className={cn('grid gap-6 lg:grid-cols-2', className)}>
      {children}
    </div>
  );
}
