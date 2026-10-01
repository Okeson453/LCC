'use client';

import * as React from 'react';
import { cn } from '@lcc/ui';

export function ReplyVariantCard({ tone, body, active, onClick }: { tone: string; body: string; active: boolean; onClick: () => void }): React.ReactElement {
  return (
    <button
      type="button"
      onClick={onClick}
      className={cn('rounded-md border p-2 text-left', active ? 'border-primary bg-primary/5' : 'hover:bg-muted/40')}
    >
      <p className="text-xs uppercase tracking-wide text-muted-foreground">{tone}</p>
      <p className="mt-1 text-xs">{body.slice(0, 100)}</p>
    </button>
  );
}
