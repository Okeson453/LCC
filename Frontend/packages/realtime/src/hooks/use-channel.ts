/**
 * useChannel — generic React hook for channel subscription.
 *
 * Returns `{ data, status, lastUpdated, reconnect }` so the UI can render
 * connection state transparently. Status mirrors `RealtimeStatus` with one
 * addition: `idle` for the pre-mount window before the subscription fires.
 */

import { useEffect, useState, useCallback } from 'react';
import type { RealtimeClient } from '../client';
import type { RealtimeStatus } from '../utils/reconnection-state';
import type { EventEnvelope } from '../envelope';

export type ChannelStatus = RealtimeStatus['status'] | 'idle';

export interface UseChannelResult<TPayload> {
  data: EventEnvelope<TPayload> | null;
  status: ChannelStatus;
  lastUpdated: number | null;
  reconnect: () => void;
}

export interface UseChannelOptions {
  /** If false, the hook subscribes but does not call `connect()` on the client. */
  autoConnect?: boolean;
  /** Override the message-type filter (defaults to channelName). */
  messageType?: string;
}

export function useChannel<TPayload = unknown>(
  client: RealtimeClient,
  channelName: string,
  options: UseChannelOptions = {},
): UseChannelResult<TPayload> {
  const [data, setData] = useState<EventEnvelope<TPayload> | null>(null);
  const [status, setStatus] = useState<ChannelStatus>('idle');
  const [lastUpdated, setLastUpdated] = useState<number | null>(null);

  const reconnect = useCallback(() => {
    client.disconnect();
    client.connect();
  }, [client]);

  useEffect(() => {
    setStatus(client.getStatus().status);
    const unsubStatus = client.subscribe(channelName, () => setStatus(client.getStatus().status));
    const unsubMessages = client.subscribe(channelName, (env) => {
      setData(env as EventEnvelope<TPayload>);
      setLastUpdated(Date.now());
    });
    if (options.autoConnect && client.getStatus().status === 'disconnected') {
      client.connect();
    }
    return () => {
      unsubStatus();
      unsubMessages();
    };
  }, [client, channelName, options.autoConnect]);

  return { data, status, lastUpdated, reconnect };
}
