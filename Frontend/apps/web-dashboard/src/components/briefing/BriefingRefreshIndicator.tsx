'use client';

import * as React from 'react';
import { RefreshCw } from 'lucide-react';
import { Button } from '@lcc/ui';

export function BriefingRefreshIndicator({ refreshing, onRefresh, lastRefreshedAt }: { refreshing: boolean; onRefresh: () => void; lastRefreshedAt: string | null }): React.ReactElement {
  return (
    <div className="flex items-center gap-2 text-xs text-muted-foreground">
      <span aria-live="polite">{refreshing ? 'Refreshing…' : lastRefreshedAt ? `Updated ${new Date(lastRefreshedAt).toLocaleTimeString()}` : 'Not yet refreshed'}</span>
      <Button size="sm" variant="ghost" onClick={onRefresh} disabled={refreshing} aria-label="Refresh briefing" type="button">
        <RefreshCw className={refreshing ? 'h-4 w-4 animate-spin' : 'h-4 w-4'} aria-hidden="true" />
      </Button>
    </div>
  );
}
