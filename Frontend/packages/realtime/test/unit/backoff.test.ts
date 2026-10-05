import { describe, expect, it } from 'vitest';
import { computeBackoffMs, type BackoffConfig } from '@lcc/realtime';

const config = (over: Partial<BackoffConfig> = {}): BackoffConfig => ({
  baseMs: 1000,
  capMs: 30_000,
  factor: 2,
  jitterRatio: 0,
  ...over,
});

/**
 * `computeBackoffMs(attempt, config)` takes a `BackoffConfig` object, not the
 * old `(attempt, baseMs, capMs, jitterMs)` positional list. These tests were
 * written against the positional form, so `config` was receiving a number:
 * `config.capMs` and `config.baseMs` were `undefined`, every delay came back
 * `NaN`, and the assertions failed on NaN comparisons.
 */
describe('backoff', () => {
  it('doubles up to a cap', () => {
    let prev = 0;
    for (let i = 0; i < 12; i += 1) {
      const v = computeBackoffMs(i, config());
      expect(v).toBeLessThanOrEqual(30_000);
      if (i > 0 && i < 6) expect(v).toBeGreaterThanOrEqual(prev);
      prev = v;
    }
  });

  it('starts at baseMs and reaches the cap', () => {
    expect(computeBackoffMs(0, config())).toBe(1000);
    expect(computeBackoffMs(1, config())).toBe(2000);
    expect(computeBackoffMs(20, config())).toBe(30_000);
  });

  it('keeps jitter within the configured ratio', () => {
    // jitterRatio is a fraction of the base delay, not an absolute millisecond
    // value: the implementation computes `base * jitterRatio * (rand*2-1)`,
    // i.e. symmetric +/- around the base.
    const base = computeBackoffMs(2, config({ baseMs: 100, jitterRatio: 0 }));
    for (let i = 0; i < 50; i += 1) {
      const jittered = computeBackoffMs(2, config({ baseMs: 100, jitterRatio: 0.5 }));
      expect(jittered).toBeGreaterThanOrEqual(base * 0.5);
      expect(jittered).toBeLessThanOrEqual(base * 1.5);
    }
  });

  it('never returns a negative delay', () => {
    expect(computeBackoffMs(0, config({ jitterRatio: 10 }))).toBeGreaterThanOrEqual(0);
  });
});
