/**
 * Channel subscription: ws/sequence
 *
 * Pushes sequence timeline updates (step sent, reply detected, paused).
 */

import type { EventEnvelope } from '../envelope';
import type { RealtimeClient } from '../client';

export interface SequenceEventPayload {
  sequenceId: string;
  stepId: string;
  action: 'scheduled' | 'sent' | 'replied' | 'paused' | 'resumed';
  /** When reply is detected, pause-reason is set. */
  pauseReason: string | null;
}

export function subscribeSequenceChannel(
  client: RealtimeClient,
  listener: (env: EventEnvelope<SequenceEventPayload>) => void,
): () => void {
  return client.subscribeByType('sequence.updated', listener as never);
}

export const SEQUENCE_CHANNEL_PATH = '/api/v1/ws/sequence';
