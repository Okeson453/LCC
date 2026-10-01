/**
 * OAuthExpiredBanner — token-expiry UX. Audit ref: M-15.
 */
'use client';

import * as React from 'react';
import Link from 'next/link';
import { Alert, AlertTitle, AlertDescription, Button } from '@lcc/ui';

export function OAuthExpiredBanner({ connectedAt }: { connectedAt: string | null }): React.ReactElement | null {
  // We assume expiring at 50 minutes (LinkedIn refresh window). Banner shows once
  // the session is 25 minutes old; clearer recovery CTA shown past 50 minutes.
  if (!connectedAt) return null;
  const ageMinutes = (Date.now() - new Date(connectedAt).getTime()) / 60_000;
  if (ageMinutes < 25) return null;
  const pastRefresh = ageMinutes >= 50;

  return (
    <Alert variant={pastRefresh ? 'destructive' : 'warning'} role="status">
      <AlertTitle>{pastRefresh ? 'LinkedIn session expired' : 'LinkedIn session expiring soon'}</AlertTitle>
      <AlertDescription>
        {pastRefresh
          ? 'Actions have been paused. Reconnect your LinkedIn account.'
          : 'Reconnect before your next approval to avoid disruption.'}
      </AlertDescription>
      <Button asChild size="sm" className="mt-2">
        <Link href="/settings/oauth">Reconnect</Link>
      </Button>
    </Alert>
  );
}
