/**
 * IdempotencyKeyBadge — visible idempotency_key on action surfaces.
 * Audit ref: M-14 + A-04.
 */
import * as React from 'react';
import { TooltipProvider, Tooltip, TooltipTrigger, TooltipContent } from '@lcc/ui';

export function IdempotencyKeyBadge({ keyValue }: { keyValue: string | null }): React.ReactElement | null {
  if (!keyValue) return null;
  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <span
            aria-label={`idempotency-key ${keyValue}`}
            className="ml-2 inline-flex items-center rounded bg-muted px-2 py-0.5 font-mono text-[10px] text-muted-foreground"
          >
            ik:{keyValue.slice(0, 8)}
          </span>
        </TooltipTrigger>
        <TooltipContent>{keyValue}</TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}
