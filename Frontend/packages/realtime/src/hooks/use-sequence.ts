/**
 * useSequence — typed convenience for ws/sequence.
 */

import { useChannel } from './use-channel';
import type { RealtimeClient } from '../client';
import type { SequenceEventPayload } from '../channels/sequence';

export function useSequence(client: RealtimeClient) {
  return useChannel<SequenceEventPayload>(client, 'sequence.updated', { autoConnect: true });
}
