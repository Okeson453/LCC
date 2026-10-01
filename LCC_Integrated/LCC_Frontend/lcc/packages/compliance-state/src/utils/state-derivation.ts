/**
 * State derivation utilities for the restricted-state machine.
 */

import type { RestrictionStateDTO } from '@lcc/api-types';

export type DerivedRestrictionState =
  | { kind: 'active'; reason: RestrictionStateDTO['reason']; reasonDetail: string | null; triggeredAt: string }
  | { kind: 'cleared'; clearedAt: string | null }
  | { kind: 'unknown' };

export function deriveRestrictionState(
  state: RestrictionStateDTO | null | undefined,
): DerivedRestrictionState {
  if (!state) return { kind: 'unknown' };
  if (state.is_restricted) {
    return {
      kind: 'active',
      reason: state.reason,
      reasonDetail: state.reason_detail,
      triggeredAt: state.triggered_at ?? new Date().toISOString(),
    };
  }
  return { kind: 'cleared', clearedAt: state.cleared_at };
}

export const RESTRICTION_REASON_LABEL: Record<RestrictionStateDTO['reason'], string> = {
  denial_rate: 'High denial rate',
  manual_review: 'Under manual review',
  oauth_expired: 'OAuth expired',
  governance_fail: 'Governance failure',
  abuse_signal: 'Abuse signal',
  none: 'No restriction',
};

export const RESTRICTION_REASON_DESCRIPTION: Record<RestrictionStateDTO['reason'], string> = {
  denial_rate: 'The Compliance Governor has paused your account due to a high denial rate.',
  manual_review: 'Your account is being reviewed by an operator. New actions are paused.',
  oauth_expired: 'Your LinkedIn OAuth token has expired. New actions are paused.',
  governance_fail: 'The Compliance Governor failed to evaluate. New actions are paused.',
  abuse_signal: 'Abuse signals detected. New actions are paused pending review.',
  none: 'No active restriction.',
};
