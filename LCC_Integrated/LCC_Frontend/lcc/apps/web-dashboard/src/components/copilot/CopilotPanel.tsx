'use client';

import * as React from 'react';
import { Send } from 'lucide-react';
import { Button, Input } from '@lcc/ui';
import { MessageThread } from './MessageThread';
import { CopilotQuickPrompts } from './CopilotQuickPrompts';
import { CopilotInput } from './CopilotInput';

export interface CopilotMessage {
  id: string;
  role: 'user' | 'assistant';
  body: string;
  kbRefs?: Array<{ recordId: string; title: string; category: string }>;
  proposal?: { kind: string; payload: Record<string, unknown>; requiresApproval: boolean };
  createdAt: string;
}

export function CopilotPanel(): React.ReactElement {
  const [messages, setMessages] = React.useState<CopilotMessage[]>([]);
  const [input, setInput] = React.useState('');

  return (
    <div className="flex flex-1 flex-col">
      <div className="flex-1 overflow-y-auto p-4">
        {messages.length === 0 ? (
          <CopilotQuickPrompts onPick={(p) => setInput(p)} />
        ) : (
          <MessageThread messages={messages} />
        )}
      </div>
      <CopilotInput value={input} onChange={setInput} onSubmit={(text) => {
        setMessages((prev) => [
          ...prev,
          { id: crypto.randomUUID(), role: 'user', body: text, createdAt: new Date().toISOString() },
        ]);
        setInput('');
      }} />
    </div>
  );
}
