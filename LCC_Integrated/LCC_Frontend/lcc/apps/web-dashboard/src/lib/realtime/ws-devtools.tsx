/**
 * ws-devtools — dev-only WS inspector.
 */

'use client';

import * as React from 'react';
import { Button } from '@lcc/ui';
import type { RealtimeClient } from '@lcc/realtime';

export interface WsDevtoolsProps {
  client: RealtimeClient;
}

export function WsDevtools({ client }: WsDevtoolsProps): React.ReactElement | null {
  const [open, setOpen] = React.useState(false);
  const [status, setStatus] = React.useState(() => client.getStatus());

  React.useEffect(() => {
    const unsub = client.subscribe(channelName(() => {}), () => setStatus(client.getStatus()));
    return unsub;
  }, [client]);

  if (process.env.NODE_ENV === 'production') return null;
  if (!open) {
    return (
      <Button
        size="sm"
        variant="outline"
        onClick={() => setOpen(true)}
        type="button"
        className="fixed bottom-2 right-2 z-50"
      >
        WS DevTools
      </Button>
    );
  }

  return (
    <div className="fixed bottom-2 right-2 z-50 w-80 rounded-md border bg-background p-3 shadow-lg">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium">WebSocket</h3>
        <Button size="sm" variant="ghost" onClick={() => setOpen(false)} type="button">
          Close
        </Button>
      </div>
      <dl className="mt-2 space-y-1 text-xs font-mono">
        <div>
          <dt className="inline text-muted-foreground">transport: </dt>
          <dd className="inline">{status.transport}</dd>
        </div>
        <div>
          <dt className="inline text-muted-foreground">status: </dt>
          <dd className="inline">{status.status}</dd>
        </div>
        <div>
          <dt className="inline text-muted-foreground">attempt: </dt>
          <dd className="inline">{status.attempt}</dd>
        </div>
        {status.reason ? (
          <div>
            <dt className="inline text-muted-foreground">reason: </dt>
            <dd className="inline">{status.reason}</dd>
          </div>
        ) : null}
      </dl>
      <div className="mt-2 flex gap-2">
        <Button size="sm" variant="outline" onClick={() => client.connect()} type="button">
          Connect
        </Button>
        <Button size="sm" variant="ghost" onClick={() => client.disconnect()} type="button">
          Disconnect
        </Button>
      </div>
    </div>
  );
}

// Helper to avoid a chicken-and-egg with subscribe signature
function channelName(_: () => void) {
  return '__ws_status__';
}
