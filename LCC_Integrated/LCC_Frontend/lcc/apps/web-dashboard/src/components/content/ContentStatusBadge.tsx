'use client';

import * as React from 'react';
import { Badge } from '@lcc/ui';

export type ContentStatus = 'draft' | 'pending_approval' | 'approved' | 'scheduled' | 'published' | 'rejected';

export function ContentStatusBadge({ status }: { status: ContentStatus }): React.ReactElement {
  const variant: 'secondary' | 'warning' | 'success' | 'destructive' | 'outline' =
    status === 'published' ? 'success'
    : status === 'scheduled' ? 'secondary'
    : status === 'pending_approval' || status === 'draft' ? 'warning'
    : status === 'rejected' ? 'destructive'
    : 'outline';
  return <Badge variant={variant} className="capitalize">{status.replace(/_/g, ' ')}</Badge>;
}
