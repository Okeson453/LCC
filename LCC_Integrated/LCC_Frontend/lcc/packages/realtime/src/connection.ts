/**
 * WebSocket connection lifecycle — connect, reconnect with exponential
 * backoff, heartbeat, hand off to SSE fallback on repeated failures.
 */

import { Backoff, DEFAULT_BACKOFF, type BackoffConfig } from './utils/backoff';
import { Heartbeat } from './utils/heartbeat';
import { ReconnectionState, type RealtimeStatus } from './utils/reconnection-state';
import { isPingFrame, isPongFrame, type EventEnvelope } from './envelope';

export interface WsConnectionOptions {
  url: string;
  /** Auth — typically a short-lived JWT or session cookie credential. */
  token: string;
  /** Optional protocols (e.g., ['soap', 'wamp']). */
  protocols?: string | string[];
  /** Backoff configuration. Defaults to DEFAULT_BACKOFF. */
  backoff?: BackoffConfig;
  /** Heartbeat config. */
  heartbeat?: {
    intervalMs: number;
    timeoutMs: number;
  };
  /** Number of consecutive failures before falling back to SSE. */
  sseThreshold?: number;
  onMessage: (envelope: EventEnvelope<unknown>) => void;
  onStatusChange: (status: RealtimeStatus) => void;
  /** Override the WebSocket constructor (useful for tests). */
  WebSocketImpl?: typeof WebSocket;
}

export class WsConnection {
  private socket: WebSocket | null = null;
  private backoff: Backoff;
  private heartbeat: Heartbeat | null = null;
  private consecutiveFailures = 0;
  private destroyed = false;

  readonly state: ReconnectionState;

  constructor(private readonly opts: WsConnectionOptions) {
    this.state = new ReconnectionState();
    this.backoff = new Backoff(opts.backoff ?? DEFAULT_BACKOFF);
  }

  connect(): void {
    if (this.destroyed) return;
    this.state.update({
      transport: 'ws',
      status: 'connecting',
      attempt: this.backoff.current,
    });
    this.opts.onStatusChange(this.state.current);

    const WS = this.opts.WebSocketImpl ?? (globalThis as { WebSocket?: typeof WebSocket }).WebSocket;
    if (!WS) {
      this.handleFailure('WebSocket unavailable in this environment');
      return;
    }

    const protocols = this.opts.protocols;
    try {
      this.socket = protocols ? new WS(this.opts.url, protocols) : new WS(this.opts.url);
    } catch (e) {
      this.handleFailure((e as Error).message);
      return;
    }

    this.socket.onopen = () => this.handleOpen();
    this.socket.onmessage = (ev: MessageEvent) => this.handleMessage(ev);
    this.socket.onerror = (ev: Event) => this.handleError((ev as ErrorEvent).message ?? 'error');
    this.socket.onclose = (ev: CloseEvent) => this.handleClose(ev.code, ev.reason);
  }

  send(frame: unknown): boolean {
    if (!this.socket || this.socket.readyState !== WebSocket.OPEN) return false;
    this.socket.send(typeof frame === 'string' ? frame : JSON.stringify(frame));
    return true;
  }

  destroy(): void {
    this.destroyed = true;
    this.heartbeat?.stop();
    if (this.socket) {
      try {
        this.socket.close();
      } catch {
        /* ignore */
      }
    }
    this.state.update({ status: 'disconnected' });
  }

  private handleOpen(): void {
    this.backoff.reset();
    this.consecutiveFailures = 0;
    this.state.update({
      transport: 'ws',
      status: 'connected',
      lastConnectedAt: Date.now(),
      reason: null,
    });
    this.opts.onStatusChange(this.state.current);

    const hb = this.opts.heartbeat ?? { intervalMs: 25_000, timeoutMs: 60_000 };
    this.heartbeat = new Heartbeat({
      intervalMs: hb.intervalMs,
      timeoutMs: hb.timeoutMs,
      send: (frame) => this.send(frame),
      onTimeout: () => this.handleClose(4000, 'heartbeat_timeout'),
    });
    this.heartbeat.start();
  }

  private handleMessage(ev: MessageEvent): void {
    this.heartbeat?.recordFrameReceived();
    const raw = typeof ev.data === 'string' ? ev.data : '';
    if (!raw) return;
    if (isPingFrame(raw)) {
      this.send({ type: 'pong', ts: Date.now() });
      return;
    }
    if (isPongFrame(raw)) {
      return; // heartbeat handled
    }
    try {
      const env = JSON.parse(raw) as EventEnvelope<unknown>;
      this.opts.onMessage(env);
    } catch (e) {
      this.handleFailure('Failed to parse envelope: ' + (e as Error).message);
    }
  }

  private handleError(reason: string): void {
    this.handleFailure(reason);
  }

  private handleClose(code: number, reason: string): void {
    this.heartbeat?.stop();
    this.state.update({
      status: 'reconnecting',
      lastDisconnectedAt: Date.now(),
      reason: `code=${code} reason=${reason}`,
    });
    this.opts.onStatusChange(this.state.current);
    this.scheduleReconnect();
  }

  private handleFailure(reason: string): void {
    this.consecutiveFailures += 1;
    const threshold = this.opts.sseThreshold ?? 5;
    if (this.consecutiveFailures >= threshold) {
      this.state.update({ transport: 'sse', status: 'degraded', reason });
      this.opts.onStatusChange(this.state.current);
      // The RealtimeClient (caller) is responsible for actually opening the
      // SSE connection when transport flips to 'sse'. We just signal.
      this.destroy();
      return;
    }
    this.state.update({ reason, status: 'reconnecting' });
    this.opts.onStatusChange(this.state.current);
    this.scheduleReconnect();
  }

  private scheduleReconnect(): void {
    const delay = this.backoff.next();
    setTimeout(() => {
      if (!this.destroyed) this.connect();
    }, delay);
  }
}
