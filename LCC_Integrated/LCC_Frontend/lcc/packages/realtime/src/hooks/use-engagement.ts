/**
 * useEngagement — typed convenience for ws/engagement.
 */

import { useChannel } from './use-channel';
import type { RealtimeClient } from '../client';
import type { EngagementEventPayload } from '../channels/engagement';

export function useEngagement(client: RealtimeClient) {
  return useChannel<EngagementEventPayload>(client, 'engagement.refresh', { autoConnect: true });
}
