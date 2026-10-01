'use client';

import * as React from 'react';
import { TooltipProvider, Tooltip, TooltipTrigger, TooltipContent, Badge } from '@lcc/ui';

export function ContactWarmthBadge({ warmth }: { warmth: number }): React.ReactElement {
  const tone = warmth > 0.66 ? 'success' : warmth > 0.33 ? 'secondary' : 'outline';
  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <Badge variant={tone}>{Math.round(warmth * 100)}°</Badge>
        </TooltipTrigger>
        <TooltipContent>Warmth score (0–100)</TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}
