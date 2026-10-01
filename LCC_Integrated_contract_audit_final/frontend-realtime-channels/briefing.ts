/**
 * Channel subscription: ws/briefing
 *
 * Subscribes to daily briefing refresh events. The hook consumer (web app)
 * wires this into TanStack Query invalidation.
 */

import type { EventEnvelope } from '../envelope';
import type { RealtimeClient } from '../client';

export interface BriefingRefreshPayload {
  date: string;
  /** Section counts (not full payload — consumer refetches). */
  approvalsDue: number;
  hotOpportunities: number;
  engagement: number;
  followups: number;
}

export function subscribeBriefingChannel(
  client: RealtimeClient,
  listener: (env: EventEnvelope<BriefingRefreshPayload>) => void,
): () => void {
  return client.subscribeByType('briefing.refresh', listener as never);
}

export const BRIEFING_CHANNEL_PATH = '/api/v1/ws/briefing';
