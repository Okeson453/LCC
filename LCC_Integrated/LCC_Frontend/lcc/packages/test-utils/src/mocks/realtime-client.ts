/**
 * Mock RealtimeClient for tests.
 *
 * Implements the same interface as the production client but never opens
 * a real WebSocket. Tests can emit envelopes directly.
 */

import type { RealtimeClient } from '@lcc/realtime';
import type { EventEnvelope, RealtimeMessageType } from '@lcc/realtime';

export function createMockRealtimeClient(): RealtimeClient & {
  emit: (env: EventEnvelope<unknown>) => void;
} {
  const channelListeners = new Map<string, Set<(env: EventEnvelope<unknown>) => void>>();
  const typeListeners = new Map<RealtimeMessageType, Set<(env: EventEnvelope<unknown>) => void>>();

  const client = {
    connect() {},
    disconnect() {},
    getStatus() {
      return {
        transport: 'none' as const,
        status: 'disconnected' as const,
        attempt: 0,
        lastConnectedAt: null,
        lastDisconnectedAt: null,
        reason: null,
      };
    },
    subscribe(channelName: string, listener: (env: EventEnvelope<unknown>) => void) {
      let set = channelListeners.get(channelName);
      if (!set) {
        set = new Set();
        channelListeners.set(channelName, set);
      }
      set.add(listener);
      return () => set!.delete(listener);
    },
    subscribeByType(type: RealtimeMessageType, listener: (env: EventEnvelope<unknown>) => void) {
      let set = typeListeners.get(type);
      if (!set) {
        set = new Set();
        typeListeners.set(type, set);
      }
      set.add(listener);
      return () => set!.delete(listener);
    },
    emit(env: EventEnvelope<unknown>) {
      const ch = channelListeners.get(env.event_type);
      if (ch) for (const l of ch) l(env);
      const tp = typeListeners.get(env.event_type as RealtimeMessageType);
      if (tp) for (const l of tp) l(env);
    },
  };
  return client as never;
}
