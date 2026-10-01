import { describe, expect, it } from 'vitest';
import {
  memberId,
  isMemberId,
  traceId,
  isTraceId,
  idempotencyKey,
  isIdempotencyKey,
  permitToken,
  generateIdempotencyKey,
  generateTraceId,
} from '../src/runtime/brand';

describe('brand types', () => {
  describe('memberId', () => {
    it('accepts a valid UUID', () => {
      const id = memberId('0d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f');
      expect(id).toBe('0d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f');
    });

    it('rejects an invalid UUID', () => {
      expect(() => memberId('not-a-uuid')).toThrow(TypeError);
    });

    it('type-guards via isMemberId', () => {
      expect(isMemberId('0d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f')).toBe(true);
      expect(isMemberId('nope')).toBe(false);
    });
  });

  describe('traceId', () => {
    it('accepts a UUID v4', () => {
      const id = traceId('0d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f');
      expect(id).toBe('0d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f');
    });

    it('rejects a UUID v1 (non-v4)', () => {
      expect(() => traceId('57c6a4d0-7e6e-11ec-b909-0242ac120002')).toThrow(TypeError);
    });

    it('type-guards via isTraceId', () => {
      expect(isTraceId('0d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f')).toBe(true);
    });
  });

  describe('idempotencyKey', () => {
    it('accepts a UUID', () => {
      const k = idempotencyKey('0d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f');
      expect(k).toBe('0d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f');
    });

    it('rejects too-short keys', () => {
      expect(() => idempotencyKey('abc')).toThrow(TypeError);
    });
  });

  describe('permitToken', () => {
    it('accepts long opaque strings', () => {
      const t = permitToken('pt_abcdef1234567890abcdef1234567890');
      expect(t.length).toBeGreaterThanOrEqual(16);
    });

    it('rejects short tokens', () => {
      expect(() => permitToken('short')).toThrow(TypeError);
    });
  });

  describe('generators', () => {
    it('generateTraceId returns a valid UUID v4', () => {
      const t = generateTraceId();
      expect(isTraceId(t)).toBe(true);
    });

    it('generateIdempotencyKey returns a valid key', () => {
      const k = generateIdempotencyKey();
      expect(isIdempotencyKey(k)).toBe(true);
    });
  });
});
