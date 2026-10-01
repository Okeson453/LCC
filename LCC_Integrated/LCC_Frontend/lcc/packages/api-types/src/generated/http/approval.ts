/* eslint-disable */
/**
 * @generated
 * From schemas/openapi/api-gateway.yaml — Approval resources.
 */

import type { UUID, DateTime } from './common';
import type { KbCitationRef } from './profile';

export type ApprovalStatus = 'pending' | 'approved' | 'rejected' | 'expired' | 'failed';
export type ApprovalActionType =
  | 'publish_post'
  | 'send_connection'
  | 'send_dm'
  | 'send_message'
  | 'apply_opportunity'
  | 'send_proposal'
  | 'edit_profile'
  | 'comment'
  | 'like';

export interface GovernanceRecord {
  permit: boolean;
  failed_guard: string;
  reason: string;
}

export interface Approval {
  id: UUID;
  member_id: UUID;
  action_type: ApprovalActionType;
  tier: 1 | 2 | 3 | 4 | 5;
  status: ApprovalStatus;
  payload: Record<string, unknown>;
  kb_refs: KbCitationRef[];
  governance: GovernanceRecord | null;
  trace_id: UUID;
  idempotency_key: string;
  created_at: DateTime;
  decided_at: DateTime | null;
}

export interface ApprovalDecision {
  decision: 'approve' | 'reject';
  edited_payload?: Record<string, unknown>;
  comment?: string;
}

export interface GuardFailure {
  guard: string;
  reason: string;
  trace_id: UUID;
}
