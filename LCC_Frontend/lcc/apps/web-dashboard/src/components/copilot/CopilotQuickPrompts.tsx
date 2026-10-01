'use client';

import * as React from 'react';
import { Button } from '@lcc/ui';

const PROMPTS = [
  'Optimize my headline for CTO roles',
  'Find 10 fintech CTOs who raised Series A in the last 90 days',
  'Summarize this week\'s activity',
  'Draft a proposal for my top opportunity',
];

export interface CopilotQuickPromptsProps {
  onPick: (text: string) => void;
  className?: string;
}

export function CopilotQuickPrompts({ onPick, className }: CopilotQuickPromptsProps): React.ReactElement {
  return (
    <div className={className}>
      <p className="text-sm text-muted-foreground">Try one of these to get started:</p>
      <ul className="mt-3 space-y-2">
        {PROMPTS.map((p) => (
          <li key={p}>
            <Button
              variant="outline"
              size="sm"
              onClick={() => onPick(p)}
              type="button"
              className="w-full justify-start text-left"
            >
              {p}
            </Button>
          </li>
        ))}
      </ul>
    </div>
  );
}
