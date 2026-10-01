import { describe, expect, it } from 'vitest';
import { approvalQueryKeys } from '../src/hooks/use-approval-queue';

describe('approvalQueryKeys', () => {
  it('exposes a stable root key', () => {
    expect(approvalQueryKeys.all).toEqual(['approvals']);
  });

  it('produces deterministic list keys with filter', () => {
    expect(approvalQueryKeys.list({ status: 'pending' })).toEqual([
      'approvals',
      'list',
      { status: 'pending' },
    ]);
  });

  it('produces deterministic detail keys', () => {
    expect(approvalQueryKeys.detail('abc')).toEqual(['approvals', 'detail', 'abc']);
  });

  it('different filters produce different keys', () => {
    const a = approvalQueryKeys.list({ status: 'pending' });
    const b = approvalQueryKeys.list({ status: 'decided' });
    expect(a).not.toEqual(b);
  });
});
