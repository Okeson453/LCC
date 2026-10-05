import { describe, expect, it } from 'vitest';
import { getTierUxRule } from '@lcc/approval-gate';
import type { RiskTier } from '@lcc/api-types';

/**
 * These assertions target the API `@lcc/approval-gate` actually exports.
 *
 * The previous version imported `canAutoApprove`, `requiresApproval` and
 * `requiresTypedConfirmation`, none of which the package provides. Tier UX is
 * derived from a single `getTierUxRule(tier)` function (Frontend Design
 * Concept §21.3), so the suite drives that instead.
 */
describe('tier rules', () => {
  it('resolves a UX rule for every tier', () => {
    for (const tier of [1, 2, 3, 4, 5] as RiskTier[]) {
      expect(getTierUxRule(tier)).toBeDefined();
      expect(getTierUxRule(tier).rule).toEqual(expect.any(String));
    }
  });

  it('tier-1 auto-approves without a dialog', () => {
    const r = getTierUxRule(1);
    expect(r.rule).toBe('no-dialog');
    expect(r.trapFocus).toBe(false);
    expect(r.editableMessage).toBe(false);
    expect(r.requireTypedConfirmation).toBe(false);
  });

  it('tier-2 requires a click-confirm, not a typed confirmation', () => {
    const r = getTierUxRule(2);
    expect(r.rule).toBe('single-click-confirm');
    expect(r.allowEnterSubmit).toBe(true);
    expect(r.requireTypedConfirmation).toBe(false);
  });

  it('tiers 3 and 4 allow an editable message', () => {
    expect(getTierUxRule(3).editableMessage).toBe(true);
    expect(getTierUxRule(4).editableMessage).toBe(true);
  });

  it('tier-4 warns when there is no prior interaction', () => {
    expect(getTierUxRule(4).warnNoPriorInteraction).toBe(true);
    expect(getTierUxRule(3).warnNoPriorInteraction).toBe(false);
  });

  it('tier-5 requires a typed confirmation and forbids Enter-to-send', () => {
    const r = getTierUxRule(5);
    expect(r.rule).toBe('dialog-full-preview-explicit-send');
    expect(r.requireTypedConfirmation).toBe(true);
    expect(r.allowEnterSubmit).toBe(false);
  });
});
