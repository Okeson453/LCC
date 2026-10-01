'use client';

import * as React from 'react';
import Link from 'next/link';
import { Button, Badge } from '@lcc/ui';
import { SequenceStatusBadge } from './SequenceStatusBadge';

export function SequenceHeader({ sequence, onPause }: { sequence: { name: string; status: 'draft' | 'active' | 'paused' | 'completed' | 'archived' }; onPause: () => void }): React.ReactElement {
  return (
    <header className="flex items-center justify-between">
      <div>
        <h1 className="text-2xl font-bold">{sequence.name}</h1>
        <div className="mt-1"><SequenceStatusBadge status={sequence.status} /></div>
      </div>
      <div className="flex gap-2">
        <Button asChild variant="outline" size="sm" type="button"><Link href="/outreach">Back</Link></Button>
        <Button onClick={onPause} variant="outline" disabled={sequence.status !== 'active'} type="button">Pause</Button>
      </div>
    </header>
  );
}
