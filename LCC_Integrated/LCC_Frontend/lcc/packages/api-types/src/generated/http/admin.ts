/* eslint-disable */
/**
 * @generated
 * From schemas/openapi/api-gateway.yaml — Admin / Compliance resources.
 */

import type { UUID, DateTime } from './common';

export type ComplianceConfigStatus = 'draft' | 'proposed' | 'active' | 'retired';
export type RestrictionReason =
  | 'denial_rate'
  | 'manual_review'
  | 'oauth_expired'
  | 'governance_fail'
  | 'abuse_signal'
  | 'none';

export interface ComplianceConfig {
  daily_action_cap?: number;
  connection_per_day_cap?: number;
  dm_per_day_cap?: number;
  min_grounding_score?: number;
  tier2_approval_required?: boolean;
  tier3_approval_required?: boolean;
}

export interface ComplianceReviewer {
  reviewer_id: string;
  decision: 'pending' | 'approved' | 'rejected';
  decided_at: DateTime | null;
}

export interface ComplianceConfigVersion {
  id: UUID;
  version: number;
  status: ComplianceConfigStatus;
  config: ComplianceConfig;
  reviewers: ComplianceReviewer[];
  created_at: DateTime;
  activated_at: DateTime | null;
}

export interface ComplianceConfigCreate {
  config: ComplianceConfig;
  rationale: string;
}

export interface RestrictionState {
  member_id: UUID;
  is_restricted: boolean;
  reason: RestrictionReason;
  reason_detail: string | null;
  triggered_at: DateTime | null;
  cleared_at: DateTime | null;
}
