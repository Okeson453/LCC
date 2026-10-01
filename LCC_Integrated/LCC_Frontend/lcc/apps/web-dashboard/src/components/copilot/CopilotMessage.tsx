'use client';

import * as React from 'react';
import { cn } from '@lcc/ui';
import { CopilotMessage } from './CopilotPanel';
import { ActionProposalCard } from './ActionProposalCard';
import { KbCitationChip } from './KbCitationChip';

export interface CopilotMessageViewProps {
  message: CopilotMessage;
  className?: string;
}

export function CopilotMessageView({ message, className }: CopilotMessageViewProps): React.ReactElement {
  const isUser = message.role === 'user';
  return (
    <article
      className={cn('my-2 flex', isUser ? 'justify-end' : 'justify-start', className)}
      aria-label={isUser ? 'You' : 'Copilot'}
    >
      <div
        className={cn(
          'max-w-[85%] rounded-lg px-3 py-2 text-sm',
          isUser ? 'bg-primary text-primary-foreground' : 'bg-muted',
        )}
      >
        <p>{message.body}</p>
        {message.kbRefs && message.kbRefs.length > 0 ? (
          <div className="mt-2 flex flex-wrap gap-1">
            {message.kbRefs.map((k) => (
              <KbCitationChip key={k.recordId} citation={k} />
            ))}
          </div>
        ) : null}
        {message.proposal ? (
          <ActionProposalCard proposal={message.proposal} />
        ) : null}
      </div>
    </article>
  );
}
