import { describe, expect, it } from 'vitest';
import { Backoff, DEFAULT_BACKOFF } from '../src/utils/backoff';

describe('Backoff', () => {
  it('uses the configured base for the first attempt', () => {
    const b = new Backoff({ ...DEFAULT_BACKOFF, baseMs: 100, capMs: 10_000 }, () => 0.5);
    expect(b.next()).toBe(100);
  });

  it('doubles up to cap', () => {
    const b = new Backoff({ ...DEFAULT_BACKOFF, baseMs: 100, capMs: 800, jitterRatio: 0 }, () => 0.5);
    expect(b.next()).toBe(100);
    expect(b.next()).toBe(200);
    expect(b.next()).toBe(400);
    expect(b.next()).toBe(800);
    expect(b.next()).toBe(800); // capped
  });

  it('applies jitter within configured ratio', () => {
    // `Backoff.next()` doubles the base on every call, so the second attempt's
    // base is 2000 and its jittered ceiling is 2000 * 1.25 = 2500. The old
    // assertions compared that value against the *first* attempt's 1250
    // ceiling, which could never hold.
    const b = new Backoff({ ...DEFAULT_BACKOFF, baseMs: 1000, jitterRatio: 0.25 }, () => 1); // max positive
    expect(b.next()).toBe(1250);
    expect(b.next()).toBe(2500);

    const b2 = new Backoff({ ...DEFAULT_BACKOFF, baseMs: 1000, jitterRatio: 0.25 }, () => 0); // max negative
    expect(b2.next()).toBe(750);
    expect(b2.next()).toBe(1500);
  });

  it('resets the attempt counter', () => {
    const b = new Backoff({ ...DEFAULT_BACKOFF, baseMs: 100, jitterRatio: 0 }, () => 0.5);
    b.next();
    b.next();
    b.reset();
    expect(b.next()).toBe(100);
  });

  it('returns 0 if base * jitter overshoots', () => {
    const b = new Backoff({ ...DEFAULT_BACKOFF, baseMs: 100, jitterRatio: 1.0 }, () => 0);
    expect(b.next()).toBe(0);
  });
});
