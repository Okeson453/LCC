/**
 * Polling fallback — last resort when both WS and SSE are unavailable.
 *
 * Polls `url` at `intervalMs`. Respects exponential backoff on errors.
 * Once any other transport becomes available, polling should be torn down
 * by the parent `RealtimeClient`.
 */

import type { RealtimeStatus } from './utils/reconnection-state';
import { Backoff, DEFAULT_BACKOFF, type BackoffConfig } from './utils/backoff';

export interface PollingOptions {
  url: string;
  token: string;
  intervalMs: number;
  backoff?: BackoffConfig;
  onUpdate: (data: unknown) => void;
  onStatusChange: (status: RealtimeStatus) => void;
  /** Override fetch (useful for tests). */
  fetchImpl?: typeof fetch;
}

export class PollingConnection {
  private timer: ReturnType<typeof setTimeout> | null = null;
  private backoff: Backoff;
  private destroyed = false;

  constructor(private readonly opts: PollingOptions) {
    this.backoff = new Backoff(opts.backoff ?? { ...DEFAULT_BACKOFF, capMs: 120_000 });
  }

  start(): void {
    if (this.destroyed) return;
    this.tick();
  }

  destroy(): void {
    this.destroyed = true;
    if (this.timer) clearTimeout(this.timer);
  }

  private async tick(): Promise<void> {
    if (this.destroyed) return;
    const fetchImpl = this.opts.fetchImpl ?? globalThis.fetch.bind(globalThis);
    try {
      const res = await fetchImpl(this.opts.url, {
        headers: { Authorization: `Bearer ${this.opts.token}` },
      });
      if (!res.ok) {
        this.handleFailure(`HTTP ${res.status}`);
        return;
      }
      const data = await res.json();
      this.opts.onUpdate(data);
      this.backoff.reset();
      this.opts.onStatusChange({
        transport: 'polling',
        status: 'connected',
        attempt: 0,
        lastConnectedAt: Date.now(),
        lastDisconnectedAt: null,
        reason: null,
      });
      this.timer = setTimeout(() => this.tick(), this.opts.intervalMs);
    } catch (e) {
      this.handleFailure((e as Error).message);
    }
  }

  private handleFailure(reason: string): void {
    this.opts.onStatusChange({
      transport: 'polling',
      status: 'reconnecting',
      attempt: this.backoff.current,
      lastConnectedAt: null,
      lastDisconnectedAt: Date.now(),
      reason,
    });
    const delay = this.backoff.next();
    this.timer = setTimeout(() => this.tick(), delay);
  }
}
