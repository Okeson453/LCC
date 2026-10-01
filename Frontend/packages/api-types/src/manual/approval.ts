/**
 * Approval types — manual additions to the OpenAPI-generated Approval shape.
 *
 * The base Approval type comes from codegen (generated/http/approval.ts).
 * Here we define the UI-facing view models and decision types.
 */

import type { KbCitation } from './kb-citation';
import type { RiskTier } from './risk-tier';

/** Approval status — matches OpenAPI enum. */
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

export const ACTION_TYPE_LABELS: Record<ApprovalActionType, string> = {
  publish_post: 'Publish post',
  send_connection: 'Send connection request',
  send_dm: 'Send DM',
  send_message: 'Send outreach message',
  apply_opportunity: 'Apply to opportunity',
  send_proposal: 'Send proposal',
  edit_profile: 'Edit profile',
  comment: 'Post comment',
  like: 'React',
};

export interface GovernanceDecision {
  permit: boolean;
  failedGuard: string | null;
  reason: string | null;
}

export interface ApprovalViewModel {
  id: string;
  memberId: string;
  actionType: ApprovalActionType;
  tier: RiskTier;
  status: ApprovalStatus;
  /** Render-target preview (text/HTML body). */
  preview: string;
  /** Where this action targets (person, post, opportunity, profile field). */
  target: ApprovalTarget;
  kbRefs: KbCitation[];
  governance: GovernanceDecision | null;
  traceId: string;
  idempotencyKey: string;
  createdAt: string;
  decidedAt: string | null;
}

export type ApprovalTarget =
  | { kind: 'post'; postId: string; title: string | null }
  | { kind: 'connection'; personId: string; displayName: string }
  | { kind: 'message'; personId: string; displayName: string }
  | { kind: 'opportunity'; opportunityId: string; title: string }
  | { kind: 'profile'; field: string }
  | { kind: 'comment'; postId: string; authorName: string }
  | { kind: 'unknown' };

/** Decision payload posted to /members/{id}/approvals/{approvalId}/decide. */
export interface ApprovalDecisionPayload {
  decision: 'approve' | 'reject';
  editedPayload?: Record<string, unknown>;
  comment?: string;
}

export interface ApprovalDecisionResponse {
  approval: ApprovalViewModel;
  governance: GovernanceDecision;
  traceId: string;
}
