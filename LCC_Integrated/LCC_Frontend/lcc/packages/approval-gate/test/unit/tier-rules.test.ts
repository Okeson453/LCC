import { describe, expect, it } from 'vitest';
import { canAutoApprove, requiresTypedConfirmation, requiresApproval } from '@lcc/approval-gate';

describe('tier rules', () => {
  it('rejects unknown tiers', () => {
    expect(requiresApproval(0 as unknown as 1 | 2 | 3 | 4 | 5)).toBe(true);
  });

  it('tier-1 auto-approves', () => {
    expect(canAutoApprove(1)).toBe(true);
    expect(requiresApproval(1)).toBe(false);
  });

  it('tier-3+ requires typed confirmation', () => {
    expect(requiresApproval(3)).toBe(true);
    expect(requiresTypedConfirmation(3)).toBe(true);
  });

  it('tier-5 requires typed confirmation', () => {
    expect(requiresTypedConfirmation(5)).toBe(true);
  });

  it('tier-2 requires approval, no typed confirmation', () => {
    expect(requiresApproval(2)).toBe(true);
    expect(requiresTypedConfirmation(2)).toBe(false);
  });
});
