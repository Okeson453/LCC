/**
 * Exponential backoff with jitter.
 *
 * Used for WS / SSE / polling reconnection. The first delay is `baseMs`,
 * subsequent delays double up to `capMs`, with a ±25% random jitter to avoid
 * thundering-herd reconnects.
 */

export function computeBackoffMs(attempt: number, config: BackoffConfig = DEFAULT_BACKOFF): number {
  const base = Math.min(config.capMs, config.baseMs * Math.pow(config.factor, attempt));
  const jitter = base * config.jitterRatio * (Math.random() * 2 - 1);
  return Math.max(0, Math.round(base + jitter));
}

export interface BackoffConfig {
  baseMs: number;
  capMs: number;
  factor: number;
  jitterRatio: number;
}

export const DEFAULT_BACKOFF: BackoffConfig = {
  baseMs: 500,
  capMs: 30_000,
  factor: 2,
  jitterRatio: 0.25,
};

export class Backoff {
  private attempt = 0;

  constructor(
    private readonly config: BackoffConfig = DEFAULT_BACKOFF,
    private readonly rng: () => number = Math.random,
  ) {}

  reset(): void {
    this.attempt = 0;
  }

  /**
   * Returns the next delay in milliseconds. The caller is expected to
   * `setTimeout(delay)` and then re-invoke `next()` on the next failure.
   */
  next(): number {
    const base = Math.min(
      this.config.capMs,
      this.config.baseMs * Math.pow(this.config.factor, this.attempt),
    );
    const jitter = base * this.config.jitterRatio * (this.rng() * 2 - 1);
    this.attempt += 1;
    return Math.max(0, Math.round(base + jitter));
  }

  get current(): number {
    return this.attempt;
  }
}
