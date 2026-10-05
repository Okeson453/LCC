import { describe, expect, it } from 'vitest';
import {
  generateUuidV4,
  isMemberId,
  isTraceId,
  memberId,
  traceId,
} from '@lcc/api-types';

/**
 * The brand helpers validate their input: `memberId`/`asMemberId` require a
 * UUID-shaped string and `traceId`/`asTraceId` require a UUID *v4*.
 *
 * This suite previously used ids like `'mem-1'` and `'xxx'`, which those
 * validators reject, so every case failed with
 * `TypeError: Invalid MemberId: expected UUID, got "mem-1"` — the tests were
 * written against a pre-validation version of the module and had never been
 * run (vitest was not resolvable from this package).
 */
const UUID = '11111111-2222-4333-8444-555555555555';
const UUID_V4 = generateUuidV4();

describe('branded types', () => {
  it('produces distinct branded values', () => {
    const m = memberId(UUID);
    expect(m).toBe(UUID);
    expect(isMemberId(m)).toBe(true);
    // A non-UUID is not a MemberId.
    expect(isMemberId('mem-1')).toBe(false);
  });

  it('brand helpers throw on invalid input', () => {
    // There is no separate `as*` variant: the brand functions
    // (`memberId`, `traceId`, …) validate and throw themselves.
    expect(() => traceId(UUID_V4)).not.toThrow();
    expect(() => traceId('xxx')).toThrow();
    expect(() => memberId('not-a-member' as unknown as never)).toThrow();
  });

  it('distinguishes a UUID v4 (trace id) from a plain UUID (member id)', () => {
    // `UUID` is v4-shaped, so it is valid for both guards.
    expect(isMemberId(UUID)).toBe(true);
    expect(isTraceId(UUID)).toBe(true);
    // A version-1 UUID is a valid MemberId but not a valid TraceId: the trace
    // guard is deliberately stricter.
    const UUID_V1 = '11111111-2222-1333-8444-555555555555';
    expect(isMemberId(UUID_V1)).toBe(true);
    expect(isTraceId(UUID_V1)).toBe(false);
  });

  it('generates trace ids that satisfy the brand guard', () => {
    expect(isTraceId(UUID_V4)).toBe(true);
  });
});
