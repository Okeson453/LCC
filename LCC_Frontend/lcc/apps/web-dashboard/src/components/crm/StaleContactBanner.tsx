'use client';

import * as React from 'react';
import Link from 'next/link';
import { differenceInDays } from 'date-fns';
import { Alert, AlertTitle, AlertDescription, Button } from '@lcc/ui';

export function StaleContactBanner({ count }: { count: number }): React.ReactElement | null {
  if (count <= 0) return null;
  return (
    <Alert variant="warning">
      <AlertTitle>{count} contacts are stale</AlertTitle>
      <AlertDescription>Reach out before the relationship cools.</AlertDescription>
      <Button asChild variant="outline" size="sm" className="mt-2" type="button">
        <Link href="/network/stale">Open stale list</Link>
      </Button>
    </Alert>
  );
}

export function daysSince(iso: string | null): number | null {
  if (!iso) return null;
  return differenceInDays(new Date(), new Date(iso));
}
