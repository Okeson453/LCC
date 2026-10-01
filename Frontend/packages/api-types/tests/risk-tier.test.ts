import { describe, expect, it } from 'vitest';
import {
  TIER_DESCRIPTORS,
  tierRequiresApproval,
  tierAllowsEdit,
  tierForActionType,
} from '../src/manual/risk-tier';

describe('risk-tier', () => {
  it('tier 1 does not require approval', () => {
    expect(tierRequiresApproval(1)).toBe(false);
    expect(tierAllowsEdit(1)).toBe(false);
  });

  it('tier 2 requires approval but does not allow edit', () => {
    expect(tierRequiresApproval(2)).toBe(true);
    expect(tierAllowsEdit(2)).toBe(false);
  });

  it('tier 3+ requires approval AND allows edit', () => {
    for (const t of [3, 4, 5] as const) {
      expect(tierRequiresApproval(t)).toBe(true);
      expect(tierAllowsEdit(t)).toBe(true);
    }
  });

  it('all five tiers have descriptors', () => {
    for (let t = 1; t <= 5; t++) {
      const d = TIER_DESCRIPTORS[t as 1 | 2 | 3 | 4 | 5];
      expect(d).toBeDefined();
      expect(d.label.length).toBeGreaterThan(0);
      expect(d.description.length).toBeGreaterThan(0);
      expect(d.uxRule.length).toBeGreaterThan(0);
    }
  });

  it('maps action types to correct tiers', () => {
    expect(tierForActionType('publish_post')).toBe(2);
    expect(tierForActionType('send_connection')).toBe(3);
    expect(tierForActionType('send_dm')).toBe(4);
    expect(tierForActionType('apply_opportunity')).toBe(5);
    expect(tierForActionType('send_proposal')).toBe(5);
  });

  it('falls back to tier 2 for unknown action types', () => {
    expect(tierForActionType('unknown_action')).toBe(2);
  });
});
