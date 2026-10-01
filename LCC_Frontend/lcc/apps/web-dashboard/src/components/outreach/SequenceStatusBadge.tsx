'use client';

import * as React from 'react';
import { Badge } from '@lcc/ui';

export function SequenceStatusBadge({ status }: { status: 'draft' | 'active' | 'paused' | 'completed' | 'archived' }): React.ReactElement {
  const variant: 'secondary' | 'success' | 'warning' | 'destructive' | 'outline' =
    status === 'active' ? 'success'
    : status === 'paused' ? 'warning'
    : status === 'completed' ? 'secondary'
    : status === 'archived' ? 'outline'
    : 'outline';
  return <Badge variant={variant} className="capitalize">{status}</Badge>;
}
