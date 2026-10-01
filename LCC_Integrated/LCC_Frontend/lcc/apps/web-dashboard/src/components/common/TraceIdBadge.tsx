/**
 * TraceIdBadge — visible trace_id on action surfaces. Audit ref: M-13 + A-04.
 */
import * as React from 'react';
import { TooltipProvider, Tooltip, TooltipTrigger, TooltipContent } from '@lcc/ui';

export function TraceIdBadge({ traceId }: { traceId: string | null }): React.ReactElement | null {
  if (!traceId) return null;
  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <span
            aria-label={`trace id ${traceId}`}
            className="ml-2 inline-flex items-center rounded bg-muted px-2 py-0.5 font-mono text-[10px] text-muted-foreground"
          >
            trace:{traceId.slice(0, 8)}
          </span>
        </TooltipTrigger>
        <TooltipContent>{traceId}</TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}
