import { describe, expect, it, vi } from 'vitest';
import { decideApproval } from '@/lib/api/approval';

describe('approval API', () => {
  it('posts decision with trace + idempotency', async () => {
    globalThis.fetch = vi.fn().mockResolvedValue({
      ok: true,
      text: () => Promise.resolve('{}'),
    }) as never;
    await decideApproval('m1', 'a1', { decision: 'approve', idempotency_key: 'ik1' });
    const args = (globalThis.fetch as unknown as ReturnType<typeof vi.fn>).mock.calls[0]!;
    const headers = args[1]?.headers as Record<string, string>;
    expect(headers['Idempotency-Key']).toBe('ik1');
    expect(headers['x-trace-id']).toBeDefined();
  });
});
