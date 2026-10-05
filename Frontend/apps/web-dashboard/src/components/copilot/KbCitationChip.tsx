'use client';

import * as React from 'react';
import { Tooltip, TooltipTrigger, TooltipContent } from '@lcc/ui';
import type { KbCitation } from '@lcc/api-types';

export interface KbCitationChipProps {
  /**
   * The chip only renders the title (and the tooltip body, when present), so it
   * requires just the identifying fields rather than a complete `KbCitation`.
   * Requiring the full shape meant a minimal `{recordId,title,category}` ref —
   * what `CopilotMessage.kbRefs` carries — could not be passed.
   */
  citation: Pick<KbCitation, 'recordId' | 'title'> &
    Partial<Pick<KbCitation, 'category' | 'excerpt' | 'url'>>;
  className?: string;
}

export function KbCitationChip({ citation, className }: KbCitationChipProps): React.ReactElement {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <button
          type="button"
          className={`inline-flex items-center gap-1 rounded-md border bg-background/80 px-1.5 py-0.5 text-xs hover:bg-background ${className ?? ''}`}
        >
          <span aria-hidden="true">📎</span>
          <span>{citation.title}</span>
        </button>
      </TooltipTrigger>
      <TooltipContent>
        <p className="font-medium">{citation.title}</p>
        <p className="text-xs text-muted-foreground">{citation.category}</p>
        {citation.excerpt ? (
          <p className="mt-1 max-w-xs text-xs">{citation.excerpt}</p>
        ) : null}
      </TooltipContent>
    </Tooltip>
  );
}
