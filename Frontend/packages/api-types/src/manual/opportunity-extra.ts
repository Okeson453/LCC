/**
 * @lcc/api-types — manually maintained opportunity-domain interfaces.
 * Generated types cover the wire payloads; these fill the gaps called out
 * in the frontend audit (missing ActionItem / OpportunityStage / OpportunityEvidence).
 */

import type { RiskTier } from './risk-tier';

export interface ActionItem {
  id: string;
  type: 'research' | 'outreach' | 'follow_up' | 'custom';
  title: string;
  description?: string;
  status: 'open' | 'in_progress' | 'completed' | 'blocked';
  assignee?: string;
  dueDate?: string;
  relatedOpportunityId?: string;
  relatedSequenceId?: string;
  createdAt?: string;
  updatedAt?: string;
}

export interface OpportunityStage {
  id: string;
  name: string;
  position: number;
  color?: string;
  requiredFields?: readonly string[];
}

/** Evidence backing an AI-generated opportunity or action item. */
export interface OpportunityEvidence {
  id: string;
  opportunityId?: string;
  actionItemId?: string;
  sourceUrl?: string;
  summary: string;
  confidence: number;              // 0..1
  createdAt?: string;
}

/** Stage-to-tier gate used when an opportunity advances. */
export const STAGE_TIER_THRESHOLD: Record<string, RiskTier> = {
  discovery: 3,
  qualified: 4,
  proposal: 5,
  negotiation: 5,
};
