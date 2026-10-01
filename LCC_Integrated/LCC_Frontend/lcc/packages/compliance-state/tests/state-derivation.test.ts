import { describe, expect, it } from 'vitest';
import {
  deriveRestrictionState,
  RESTRICTION_REASON_LABEL,
  RESTRICTION_REASON_DESCRIPTION,
} from '../src/utils/state-derivation';
import type { RestrictionStateDTO } from '@lcc/api-types';

const base: RestrictionStateDTO = {
  member_id: '0d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f',
  is_restricted: false,
  reason: 'none',
  reason_detail: null,
  triggered_at: null,
  cleared_at: null,
};

describe('deriveRestrictionState', () => {
  it('returns unknown for null', () => {
    expect(deriveRestrictionState(null).kind).toBe('unknown');
    expect(deriveRestrictionState(undefined).kind).toBe('unknown');
  });

  it('returns cleared when not restricted', () => {
    const d = deriveRestrictionState(base);
    expect(d.kind).toBe('cleared');
  });

  it('returns active with reason when restricted', () => {
    const restricted: RestrictionStateDTO = {
      ...base,
      is_restricted: true,
      reason: 'manual_review',
      reason_detail: 'Operator placed under review',
      triggered_at: '2026-04-22T07:00:00Z',
    };
    const d = deriveRestrictionState(restricted);
    expect(d.kind).toBe('active');
    if (d.kind === 'active') {
      expect(d.reason).toBe('manual_review');
      expect(d.reasonDetail).toBe('Operator placed under review');
    }
  });
});

describe('RESTRICTION_REASON_LABEL', () => {
  it('covers every reason', () => {
    const reasons: RestrictionStateDTO['reason'][] = [
      'denial_rate',
      'manual_review',
      'oauth_expired',
      'governance_fail',
      'abuse_signal',
      'none',
    ];
    for (const r of reasons) {
      expect(RESTRICTION_REASON_LABEL[r]).toBeDefined();
      expect(RESTRICTION_REASON_DESCRIPTION[r]).toBeDefined();
    }
  });
});
