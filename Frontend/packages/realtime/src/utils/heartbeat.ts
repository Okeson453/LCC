/**
 * Heartbeat — keep-alive pings for the WS connection.
 *
 * Sends `{ type: 'ping', ts }` frames at `intervalMs`. If no `pong` (or
 * any other frame) is received within `timeoutMs`, the connection is
 * considered dead and the caller should reconnect.
 */

export interface HeartbeatOptions {
  intervalMs: number;
  timeoutMs: number;
  send: (frame: { type: 'ping'; ts: number }) => void;
  onTimeout: () => void;
}

export class Heartbeat {
  private interval: ReturnType<typeof setInterval> | null = null;
  private timeout: ReturnType<typeof setTimeout> | null = null;
  private lastPongAt = 0;

  constructor(private readonly opts: HeartbeatOptions) {}

  start(): void {
    this.stop();
    this.lastPongAt = Date.now();
    this.interval = setInterval(() => this.tick(), this.opts.intervalMs);
  }

  stop(): void {
    if (this.interval) clearInterval(this.interval);
    if (this.timeout) clearTimeout(this.timeout);
    this.interval = null;
    this.timeout = null;
  }

  /** Mark a frame received from the server (any type counts as proof of life). */
  recordFrameReceived(): void {
    this.lastPongAt = Date.now();
    if (this.timeout) {
      clearTimeout(this.timeout);
      this.timeout = null;
    }
  }

  private tick(): void {
    const now = Date.now();
    if (now - this.lastPongAt > this.opts.timeoutMs) {
      this.opts.onTimeout();
      this.stop();
      return;
    }
    this.opts.send({ type: 'ping', ts: now });
    this.timeout = setTimeout(() => {
      this.opts.onTimeout();
      this.stop();
    }, this.opts.timeoutMs);
  }
}
