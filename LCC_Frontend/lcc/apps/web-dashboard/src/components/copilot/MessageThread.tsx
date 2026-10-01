'use client';

import * as React from 'react';
import { CopilotMessage } from './CopilotPanel';
import { CopilotMessageView } from './CopilotMessage';

export interface MessageThreadProps {
  messages: readonly CopilotMessage[];
  className?: string;
}

export function MessageThread({ messages, className }: MessageThreadProps): React.ReactElement {
  return (
    <ol className={className} aria-label="Conversation">
      {messages.map((m) => (
        <li key={m.id}>
          <CopilotMessageView message={m} />
        </li>
      ))}
    </ol>
  );
}
