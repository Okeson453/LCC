import { describe, expect, it } from 'vitest';
import { computeBackoffMs } from '@lcc/realtime';

describe('backoff', () => {
  it('doubles up to a cap', () => {
    let prev = 0;
    for (let i = 0; i < 12; i += 1) {
      const v = computeBackoffMs(i, 1000, 30_000, 0);
      expect(v).toBeLessThanOrEqual(30_000);
      if (i > 0 && i < 6) expect(v).toBeGreaterThanOrEqual(prev);
      prev = v;
    }
  });

  it('adds jitter in [0, jitterMs)', () => {
    const base = computeBackoffMs(2, 100, 10_000, 0);
    const jittered = computeBackoffMs(2, 100, 10_000, 50);
    expect(jittered).toBeGreaterThanOrEqual(base);
    expect(jittered).toBeLessThan(base + 50);
  });
});
