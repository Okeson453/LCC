// Component test entry: ensures the directory exists and the runner has at
// least one pass to confirm wiring. Real component tests live alongside.
import { describe, expect, it } from 'vitest';

describe('tests/component exists', () => {
  it('runs', () => {
    expect(true).toBe(true);
  });
});
