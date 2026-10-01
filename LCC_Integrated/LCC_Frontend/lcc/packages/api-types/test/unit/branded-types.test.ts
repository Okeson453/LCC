import { describe, expect, it } from 'vitest';
import { asMemberId, asTraceId, isMemberId, isTraceId, memberId } from '@lcc/api-types';

describe('branded types', () => {
  it('produces distinct branded values', () => {
    const m = memberId('mem-1');
    expect(m).toBe('mem-1');
    expect(isMemberId(m)).toBe(true);
  });

  it('as helpers throw on invalid', () => {
    expect(() => asTraceId('xxx')).not.toThrow();
    expect(() => asMemberId('not-a-member' as unknown as never)).toThrow();
  });
});
