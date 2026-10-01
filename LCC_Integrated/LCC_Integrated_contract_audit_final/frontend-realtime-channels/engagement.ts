/**
 * Channel subscription: ws/engagement
 *
 * Pushes inbox / queue / reply updates.
 */

import type { EventEnvelope } from '../envelope';
import type { RealtimeClient } from '../client';

export interface EngagementEventPayload {
  kind: 'inbox' | 'queue' | 'reply';
  itemId: string;
  action: 'created' | 'updated' | 'consumed';
  priority?: 'low' | 'normal' | 'high' | 'urgent';
}

export function subscribeEngagementChannel(
  client: RealtimeClient,
  listener: (env: EventEnvelope<EngagementEventPayload>) => void,
): () => void {
  return client.subscribeByType('engagement.refresh', listener as never);
}

export const ENGAGEMENT_CHANNEL_PATH = '/api/v1/ws/engagement';
