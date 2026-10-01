'use client';

import * as React from 'react';
import { Button } from '@lcc/ui';

export function OpportunityDiscoveryButton({ onDiscover, isPending }: { onDiscover: () => Promise<void>; isPending: boolean }): React.ReactElement {
  return (
    <Button onClick={() => void onDiscover()} disabled={isPending} type="button">
      {isPending ? 'Running discovery…' : 'Run discovery'}
    </Button>
  );
}
