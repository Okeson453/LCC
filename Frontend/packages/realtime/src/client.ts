/**
 * RealtimeClient — singleton that orchestrates WS → SSE → polling fallback.
 *
 * Consumers should call `getRealtimeClient()` to get a singleton instance
 * configured with their token. The client exposes a `useChannel(name)` hook
 * (via `use-channel`) for React consumers.
 *
 * This class is environment-agnostic — it works in browser windows, the
 * extension service worker, and the extension popup/sidepanel.
 */

import { WsConnection } from './connection';
import { SseConnection } from './sse-fallback';
import { PollingConnection } from './polling-fallback';
import type { RealtimeStatus } from './utils/reconnection-state';
import type { EventEnvelope, RealtimeMessageType } from './envelope';

export interface RealtimeClientOptions {
  baseUrl: string;
  /** Bearer token (short-lived JWT or session). */
  token: string;
  /** Member ID for scoped channels. */
  memberId: string;
  /** Override transports (useful for tests). */
  WebSocketImpl?: typeof WebSocket;
  EventSourceImpl?: typeof EventSource;
  fetchImpl?: typeof fetch;
}

type ChannelListener = (env: EventEnvelope<unknown>) => void;

let singleton: RealtimeClient | null = null;

export class RealtimeClient {
  private ws: WsConnection | null = null;
  private sse: SseConnection | null = null;
  private polling: PollingConnection | null = null;
  private listeners = new Map<string, Set<ChannelListener>>();
  private listenersByType = new Map<RealtimeMessageType, Set<ChannelListener>>();
  private status: RealtimeStatus = {
    transport: 'none',
    status: 'disconnected',
    attempt: 0,
    lastConnectedAt: null,
    lastDisconnectedAt: null,
    reason: null,
  };

  constructor(private readonly opts: RealtimeClientOptions) {}

  static getInstance(opts: RealtimeClientOptions): RealtimeClient {
    if (singleton && singleton.opts.token === opts.token && singleton.opts.memberId === opts.memberId && singleton.opts.baseUrl === opts.baseUrl) return singleton;
    singleton = new RealtimeClient(opts);
    return singleton;
  }

  static resetInstance(): void {
    singleton?.disconnect();
    singleton = null;
  }

  connect(): void {
    const wsUrl = this.opts.baseUrl.replace(/^http/i, 'ws') + '/ws';
    this.ws = new WsConnection({
      url: wsUrl,
      token: this.opts.token,
      WebSocketImpl: this.opts.WebSocketImpl,
      onMessage: (env) => this.dispatch(env),
      onStatusChange: (status) => this.updateStatus(status),
    });
    this.ws.connect();
  }

  disconnect(): void {
    this.ws?.destroy();
    this.sse?.destroy();
    this.polling?.destroy();
    this.ws = null;
    this.sse = null;
    this.polling = null;
    this.updateStatus({
      status: 'disconnected',
      transport: 'none',
      lastDisconnectedAt: Date.now(),
    });
  }

  getStatus(): RealtimeStatus {
    return this.status;
  }

  subscribe(channelName: string, listener: ChannelListener): () => void {
    let set = this.listeners.get(channelName);
    if (!set) {
      set = new Set();
      this.listeners.set(channelName, set);
    }
    set.add(listener);
    return () => set!.delete(listener);
  }

  subscribeByType(type: RealtimeMessageType, listener: ChannelListener): () => void {
    let set = this.listenersByType.get(type);
    if (!set) {
      set = new Set();
      this.listenersByType.set(type, set);
    }
    set.add(listener);
    return () => set!.delete(listener);
  }

  private dispatch(env: EventEnvelope<unknown>): void {
    // Dispatch by exact channel
    const channelListeners = this.listeners.get(env.event_type);
    if (channelListeners) for (const l of channelListeners) l(env);

    // Dispatch by type (used by ws-bridges in the web app)
    const typeListeners = this.listenersByType.get(env.event_type as RealtimeMessageType);
    if (typeListeners) for (const l of typeListeners) l(env);
  }

  private updateStatus(patch: Partial<RealtimeStatus>): void {
    this.status = { ...this.status, ...patch };

    // If WS flipped to sse, start the SSE connection
    if (patch.transport === 'sse' && !this.sse) {
      const sseUrl = this.opts.baseUrl + '/sse';
      this.sse = new SseConnection({
        url: sseUrl,
        token: this.opts.token,
        EventSourceImpl: this.opts.EventSourceImpl,
        onMessage: (env) => this.dispatch(env),
        onStatusChange: (s) => {
          if (s.status === 'degraded' && !this.polling) {
            this.startPolling();
          }
          this.status = { ...this.status, ...s };
        },
      });
      this.sse.connect();
    }
  }

  private startPolling(): void {
    const pollUrl = this.opts.baseUrl + `/members/${this.opts.memberId}/briefing/today`;
    this.polling = new PollingConnection({
      url: pollUrl,
      token: this.opts.token,
      intervalMs: 30_000,
      fetchImpl: this.opts.fetchImpl,
      onUpdate: (data) => {
        this.dispatch({
          event_id: '0d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f' as never,
          event_type: 'briefing.refresh',
          occurred_at: new Date().toISOString(),
          trace_id: '0d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f' as never,
          member_id: this.opts.memberId as never,
          payload: data,
        });
      },
      onStatusChange: (s) => {
        this.status = { ...this.status, ...s };
      },
    });
    this.polling.start();
  }
}

export function getRealtimeClient(opts: RealtimeClientOptions): RealtimeClient {
  return RealtimeClient.getInstance(opts);
}

