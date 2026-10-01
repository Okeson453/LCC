import { describe, expect, it } from 'vitest';
import * as tokens from '@lcc/tokens';

describe('design tokens', () => {
  it('exposes tier palette', () => {
    expect(tokens.tierPalette).toMatchObject({
      1: expect.any(String),
      2: expect.any(String),
      3: expect.any(String),
      4: expect.any(String),
      5: expect.any(String),
    });
  });

  it('exposes color and radius tokens', () => {
    expect(tokens.colorTokens).toBeDefined();
    expect(tokens.radiusTokens).toBeDefined();
  });
});
