/**
 * View-model for the briefing card on /today.
 */

import type { KbCitation, RiskTier } from '@lcc/api-types';

export interface BriefingCardViewModel {
  id: string;
  kind: 'approvals_due' | 'hot_opportunities' | 'engagement' | 'followups' | 'content_suggestions';
  title: string;
  summary: string;
  actionUrl: string;
  tier: RiskTier;
  approvalId: string | null;
  kbRefs: KbCitation[];
}
