import { describe, expect, it } from 'vitest';
import { classifyRestrictedState, DEFAULT_RESTRICTED_STATE } from '@lcc/compliance-state';

describe('restricted state', () => {
  it('default is unrestricted', () => {
    expect(DEFAULT_RESTRICTED_STATE.restricted).toBe(false);
    expect(DEFAULT_RESTRICTED_STATE.reason).toBeNull();
  });

  it('classifies banner level', () => {
    expect(classifyRestrictedState({ restricted: true, reason: 'cooldown', until: null })).toBe('warning');
    expect(classifyRestrictedState({ restricted: true, reason: 'pause', until: null })).toBe('danger');
    expect(classifyRestrictedState({ restricted: false, reason: null, until: null })).toBe('info');
  });
});
