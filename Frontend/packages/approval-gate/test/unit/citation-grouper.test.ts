import { describe, expect, it } from 'vitest';
import { groupCitationsByKind } from '@lcc/approval-gate';

describe('citation grouper', () => {
  it('groups by kind', () => {
    const groups = groupCitationsByKind([
      { record_id: 'k1', title: 'A', kind: 'authority', snippet: '...' },
      { record_id: 'k2', title: 'B', kind: 'authority', snippet: '...' },
      { record_id: 'k3', title: 'C', kind: 'bts', snippet: '...' },
    ]);
    expect(Object.keys(groups).sort()).toEqual(['authority', 'bts']);
    expect(groups['authority']?.length).toBe(2);
    expect(groups['bts']?.length).toBe(1);
  });
});
