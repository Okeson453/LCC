/**
 * SSE fallback — used when the WS connection repeatedly fails.
 *
 * SSE is unidirectional server-to-client, but we frame each event as an
 * `EventEnvelope` to keep the consumer API identical between transports.
 */

import type { RealtimeStatus } from './utils/reconnection-state';
import { Backoff, DEFAULT_BACKOFF, type BackoffConfig } from './utils/backoff';
import { parseEnvelope, type EventEnvelope } from './envelope';

export interface SseOptions {
  url: string;
  token: string;
  backoff?: BackoffConfig;
  onMessage: (env: EventEnvelope<unknown>) => void;
  onStatusChange: (status: RealtimeStatus) => void;
  /** Override EventSource (useful for tests). */
  EventSourceImpl?: typeof EventSource;
}

export class SseConnection {
  private source: EventSource | null = null;
  private backoff: Backoff;
  private destroyed = false;

  constructor(private readonly opts: SseOptions) {
    this.backoff = new Backoff(opts.backoff ?? { ...DEFAULT_BACKOFF, capMs: 60_000 });
  }

  connect(): void {
    if (this.destroyed) return;
    const ES = this.opts.EventSourceImpl ?? (globalThis as { EventSource?: typeof EventSource }).EventSource;
    if (!ES) {
      this.opts.onStatusChange({
        transport: 'sse',
        status: 'degraded',
        attempt: this.backoff.current,
        lastConnectedAt: null,
        lastDisconnectedAt: Date.now(),
        reason: 'EventSource unavailable',
      });
      return;
    }

    const url = this.opts.url + (this.opts.url.includes('?') ? '&' : '?') + 'token=' + encodeURIComponent(this.opts.token);
    this.opts.onStatusChange({
      transport: 'sse',
      status: 'connecting',
      attempt: this.backoff.current,
      lastConnectedAt: null,
      lastDisconnectedAt: null,
      reason: null,
    });

    try {
      this.source = new ES(url, { withCredentials: true });
    } catch (e) {
      this.handleFailure((e as Error).message);
      return;
    }

    this.source.onopen = () => {
      this.backoff.reset();
      this.opts.onStatusChange({
        transport: 'sse',
        status: 'connected',
        attempt: 0,
        lastConnectedAt: Date.now(),
        lastDisconnectedAt: null,
        reason: null,
      });
    };

    this.source.onmessage = (ev: MessageEvent) => {
      try {
        const env = parseEnvelope(ev.data);
        this.opts.onMessage(env);
      } catch {
        // ignore malformed
      }
    };

    this.source.onerror = () => this.handleFailure('SSE error');
  }

  destroy(): void {
    this.destroyed = true;
    this.source?.close();
  }

  private handleFailure(reason: string): void {
    this.opts.onStatusChange({
      transport: 'sse',
      status: 'reconnecting',
      attempt: this.backoff.current,
      lastConnectedAt: null,
      lastDisconnectedAt: Date.now(),
      reason,
    });
    const delay = this.backoff.next();
    setTimeout(() => {
      if (!this.destroyed) this.connect();
    }, delay);
  }
}
