import { describe, expect, it } from 'vitest';
import { getTierUxRule } from '../src/utils/tier-rules';

describe('tier-rules', () => {
  it('tier 1: no dialog, no approval', () => {
    const r = getTierUxRule(1);
    expect(r.rule).toBe('no-dialog');
    expect(r.trapFocus).toBe(false);
    expect(r.allowEnterSubmit).toBe(false);
  });

  it('tier 2: single-click + enter submit + no edit', () => {
    const r = getTierUxRule(2);
    expect(r.rule).toBe('single-click-confirm');
    expect(r.allowEnterSubmit).toBe(true);
    expect(r.editableMessage).toBe(false);
  });

  it('tier 3: editable message + enter submit', () => {
    const r = getTierUxRule(3);
    expect(r.rule).toBe('dialog-editable-message');
    expect(r.editableMessage).toBe(true);
    expect(r.allowEnterSubmit).toBe(true);
  });

  it('tier 4: editable + warn no-prior + enter submit', () => {
    const r = getTierUxRule(4);
    expect(r.warnNoPriorInteraction).toBe(true);
    expect(r.allowEnterSubmit).toBe(true);
  });

  it('tier 5: typed-confirmation + no enter submit + full preview', () => {
    const r = getTierUxRule(5);
    expect(r.requireTypedConfirmation).toBe(true);
    expect(r.allowEnterSubmit).toBe(false);
    expect(r.editableMessage).toBe(true);
    expect(r.trapFocus).toBe(true);
  });
});
