import { describe, expect, it } from 'vitest';
import { buildAuthHeaders, withIdempotencyKey } from '@/lib/api/auth-headers';

describe('auth-headers', () => {
  it('builds trace + idempotency', () => {
    const headers = buildAuthHeaders({ token: 'tk', idempotencyKey: 'ik', traceId: 'tr' });
    expect(headers.Authorization).toBe('Bearer tk');
    expect(headers['x-trace-id']).toBe('tr');
    expect(headers['Idempotency-Key']).toBe('ik');
  });

  it('omits idempotency for reads', () => {
    const headers = withIdempotencyKey({ token: 'tk', traceId: 'tr', method: 'GET', idempotencyKey: 'ik' });
    expect(headers['Idempotency-Key']).toBeUndefined();
  });

  it('attaches idempotency for writes', () => {
    const headers = withIdempotencyKey({ token: 'tk', traceId: 'tr', method: 'POST', idempotencyKey: 'ik' });
    expect(headers['Idempotency-Key']).toBe('ik');
  });
});
