'use client';

import * as React from 'react';
import { EmptyState } from '@lcc/ui';
import { CheckCircle2 } from 'lucide-react';
import { useRouter } from 'next/navigation';

export function ApprovalQueueEmptyState(): React.ReactElement {
  const router = useRouter();
  return (
    <EmptyState
      title="No pending approvals"
      description="All actions are either auto-approved or have been cleared. Nice."
      icon={<CheckCircle2 className="h-12 w-12 text-success" />}
      action={{ label: 'Back to dashboard', onClick: () => router.push('/today') }}
    />
  );
}
