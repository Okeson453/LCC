'use client';

import * as React from 'react';
import { Button } from '@lcc/ui';
import { useDevStore } from '@/lib/stores';

export function DevPanel(): React.ReactElement | null {
  const [open, setOpen] = React.useState(false);
  const dev = useDevStore();

  if (process.env.NODE_ENV === 'production') return null;

  if (!open) {
    return (
      <Button
        size="sm"
        variant="outline"
        onClick={() => setOpen(true)}
        type="button"
        className="fixed bottom-2 left-2 z-50"
      >
        Dev
      </Button>
    );
  }

  return (
    <div className="fixed bottom-2 left-2 z-50 w-72 rounded-md border bg-background p-3 shadow-lg">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium">Dev panel</h3>
        <Button size="sm" variant="ghost" onClick={() => setOpen(false)} type="button">
          Close
        </Button>
      </div>
      <div className="mt-2 space-y-2 text-xs">
        <label className="flex items-center gap-2">
          <input
            type="checkbox"
            checked={dev.mockRestricted}
            onChange={(e) => dev.setMockRestricted(e.target.checked)}
          />
          Mock restricted account
        </label>
        <label className="flex items-center gap-2">
          <input
            type="checkbox"
            checked={dev.mockLowGrounding}
            onChange={(e) => dev.setMockLowGrounding(e.target.checked)}
          />
          Mock low grounding
        </label>
        <label className="flex items-center gap-2">
          <input
            type="checkbox"
            checked={dev.mockOffline}
            onChange={(e) => dev.setMockOffline(e.target.checked)}
          />
          Mock offline
        </label>
        <Button size="sm" variant="outline" onClick={dev.reset} type="button" className="w-full">
          Reset
        </Button>
      </div>
    </div>
  );
}
