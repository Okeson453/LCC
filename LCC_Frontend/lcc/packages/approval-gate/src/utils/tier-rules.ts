/**
 * Tier-aware UX rules — derived from Frontend Design Concept §21.3.
 */

import type { RiskTier } from '@lcc/api-types';

export type ApprovalUxRule =
  | 'no-dialog'
  | 'single-click-confirm'
  | 'dialog-editable-message'
  | 'dialog-editable-no-prior'
  | 'dialog-full-preview-explicit-send';

export interface TierUxRule {
  rule: ApprovalUxRule;
  /** Whether the dialog should trap focus and require explicit dismissal. */
  trapFocus: boolean;
  /** Whether pressing Enter submits the dialog. */
  allowEnterSubmit: boolean;
  /** Whether the message body is editable. */
  editableMessage: boolean;
  /** Whether to show a warning if no prior interaction. */
  warnNoPriorInteraction: boolean;
  /** Whether to require a typed confirmation phrase (Tier 5). */
  requireTypedConfirmation: boolean;
}

export function getTierUxRule(tier: RiskTier): TierUxRule {
  switch (tier) {
    case 1:
      return {
        rule: 'no-dialog',
        trapFocus: false,
        allowEnterSubmit: false,
        editableMessage: false,
        warnNoPriorInteraction: false,
        requireTypedConfirmation: false,
      };
    case 2:
      return {
        rule: 'single-click-confirm',
        trapFocus: true,
        allowEnterSubmit: true,
        editableMessage: false,
        warnNoPriorInteraction: false,
        requireTypedConfirmation: false,
      };
    case 3:
      return {
        rule: 'dialog-editable-message',
        trapFocus: true,
        allowEnterSubmit: true,
        editableMessage: true,
        warnNoPriorInteraction: false,
        requireTypedConfirmation: false,
      };
    case 4:
      return {
        rule: 'dialog-editable-no-prior',
        trapFocus: true,
        allowEnterSubmit: true,
        editableMessage: true,
        warnNoPriorInteraction: true,
        requireTypedConfirmation: false,
      };
    case 5:
      return {
        rule: 'dialog-full-preview-explicit-send',
        trapFocus: true,
        allowEnterSubmit: false, // force explicit click
        editableMessage: true,
        warnNoPriorInteraction: false,
        requireTypedConfirmation: true,
      };
  }
}
