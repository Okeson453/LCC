/**
 * BriefingSection — groups related briefing items with a header.
 * Per audit M-02: shared components live in src/components/briefing/.
 */
'use client';

import * as React from 'react';
import { cn } from '@lcc/ui';

export interface BriefingSectionProps {
  title: string;
  description?: string;
  /** Item count shown next to the title. Rendered only when provided. */
  count?: number;
  children: React.ReactNode;
  className?: string;
}

export function BriefingSection({ title, description, count, children, className }: BriefingSectionProps): React.ReactElement {
  return (
    <section className={cn('space-y-2', className)} aria-labelledby={`brief-${title.replace(/\s+/g, '-').toLowerCase()}`}>
      <header>
        <h3 id={`brief-${title.replace(/\s+/g, '-').toLowerCase()}`} className="text-sm font-semibold">
          {title}
          {typeof count === 'number' ? (
            <span className="ml-2 text-xs font-normal text-muted-foreground">{count}</span>
          ) : null}
        </h3>
        {description ? <p className="text-xs text-muted-foreground">{description}</p> : null}
      </header>
      {children}
    </section>
  );
}
