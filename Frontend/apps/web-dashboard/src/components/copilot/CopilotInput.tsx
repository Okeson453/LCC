'use client';

import * as React from 'react';
import { Send } from 'lucide-react';
import { Button, Input } from '@lcc/ui';
import { cn } from '@lcc/ui';

export interface CopilotInputProps {
  value: string;
  onChange: (v: string) => void;
  onSubmit: (text: string) => void;
  placeholder?: string;
  disabled?: boolean;
  className?: string;
}

export function CopilotInput({
  value,
  onChange,
  onSubmit,
  placeholder = 'Ask anything…',
  disabled,
  className,
}: CopilotInputProps): React.ReactElement {
  function onKeyDown(e: React.KeyboardEvent<HTMLInputElement>) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      if (value.trim()) {
        onSubmit(value);
      }
    }
  }

  return (
    <form
      className={cn('flex items-center gap-2 border-t p-3', className)}
      onSubmit={(e) => {
        e.preventDefault();
        if (value.trim()) onSubmit(value);
      }}
    >
      <Input
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={onKeyDown}
        placeholder={placeholder}
        disabled={disabled}
        aria-label="Message Copilot"
        className="flex-1"
      />
      <Button
        type="submit"
        size="icon"
        disabled={disabled || !value.trim()}
        aria-label="Send message"
      >
        <Send className="h-4 w-4" aria-hidden="true" />
      </Button>
    </form>
  );
}
