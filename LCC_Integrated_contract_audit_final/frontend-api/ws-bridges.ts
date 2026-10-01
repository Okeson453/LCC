/**
 * WS bridges — wire RealtimeClient events into TanStack Query invalidations.
 *
 * Mounted once at the root of the (app) layout. Subscribes to all six
 * channels; each event invalidates the matching query keys.
 */

import { useEffect } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import {
  getRealtimeClient,
  type RealtimeClient,
} from '@lcc/realtime';
import type { MemberId } from '@lcc/api-types';

export interface WsBridgesOptions {
  memberId: MemberId;
  token: string;
  apiBase: string;
}

export function useWsBridges({ memberId, token, apiBase }: WsBridgesOptions): RealtimeClient {
  const queryClient = useQueryClient();
  const client = getRealtimeClient({ baseUrl: apiBase, token, memberId: memberId as string });

  useEffect(() => {
    const unsubs: Array<() => void> = [];

    unsubs.push(
      client.subscribeByType('briefing.refresh', () => {
        void queryClient.invalidateQueries({ queryKey: ['briefing', memberId] });
        void queryClient.invalidateQueries({ queryKey: ['content', 'list', memberId] });
      }),
    );

    unsubs.push(
      client.subscribeByType('approvals.refresh', () => {
        void queryClient.invalidateQueries({ queryKey: ['approvals'] });
      }),
    );

    unsubs.push(
      client.subscribeByType('engagement.refresh', () => {
        void queryClient.invalidateQueries({ queryKey: ['engagement'] });
      }),
    );

    unsubs.push(
      client.subscribeByType('compliance.restriction_changed', () => {
        void queryClient.invalidateQueries({ queryKey: ['compliance'] });
      }),
    );

    unsubs.push(
      client.subscribeByType('sequence.updated', () => {
        void queryClient.invalidateQueries({ queryKey: ['outreach'] });
      }),
    );

    client.connect();

    return () => {
      for (const u of unsubs) u();
    };
  }, [client, queryClient, memberId]);

  return client;
}
