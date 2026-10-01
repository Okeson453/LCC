import { describe, expect, it } from 'vitest';
import { briefingChannel, approvalsChannel, engagementChannel, complianceChannel, sequenceChannel, integrationChannel } from '@lcc/realtime';

describe('channel name builders', () => {
  it('formats member-scoped channels', () => {
    expect(briefingChannel({ memberId: 'm1' } as never)).toBe('briefing:m1');
    expect(approvalsChannel({ memberId: 'm1' } as never)).toBe('approvals:m1');
    expect(engagementChannel({ memberId: 'm1' } as never)).toBe('engagement:m1');
    expect(sequenceChannel({ memberId: 'm1', sequenceId: 's1' } as never)).toBe('sequence:m1:s1');
  });

  it('formats compliance + integration channels', () => {
    expect(complianceChannel()).toBe('compliance');
    expect(integrationChannel({ provider: 'linkedin' } as never)).toBe('integration:linkedin');
  });
});
