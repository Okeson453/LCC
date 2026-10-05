import { describe, expect, it } from 'vitest';
import * as tokens from '@lcc/tokens';

describe('design tokens', () => {
  it('exposes tier metadata for every risk tier', () => {
    // The package exports `tierMeta` (a Record<RiskTier, RiskTierMeta>), not a
    // `tierPalette` of plain colour strings. This test previously asserted on
    // `tierPalette`, which the package never exported, so it failed on a
    // non-existent export rather than on a real regression.
    for (const tier of [1, 2, 3, 4, 5] as const) {
      expect(tokens.tierMeta[tier]).toBeDefined();
      expect(tokens.tierMeta[tier].label).toEqual(expect.any(String));
      expect(tokens.tierColor(tier)).toEqual(expect.any(String));
    }
  });

  it('exposes color and radius tokens', () => {
    // Real exports are `colorsDark` / `colorsLight` and `radius`; the test
    // looked for `colorTokens` / `radiusTokens`, neither of which exists.
    expect(tokens.colorsDark).toBeDefined();
    expect(tokens.colorsLight).toBeDefined();
    expect(tokens.radius).toBeDefined();
    expect(tokens.radius.md).toEqual(expect.any(String));
  });
});
