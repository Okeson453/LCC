/* eslint-disable */
/**
 * @generated
 * From proto/compliance/governor.proto
 *
 * The Compliance Governor service — backend uses gRPC for high-throughput
 * governance evaluation. The frontend rarely calls this directly; it
 * receives the result embedded in HTTP responses.
 */

import type { UUID, DateTime } from '../compliance/_common';

export interface PermitToken {
  /** Opaque token issued by the governor. */
  value: string;
  /** Permitted action types (union). */
  allowedActionTypes: string[];
  /** Tier cap permitted. */
  maxTier: number;
  /** When this permit expires. */
  expiresAt: DateTime;
  /** Quota remaining under this permit. */
  quotaRemaining: number;
}

export interface GovernanceEvaluationRequest {
  memberId: UUID;
  actionType: string;
  tier: number;
  payload: Record<string, unknown>;
  kbRefs: UUID[];
  traceId: UUID;
}

export interface GovernanceEvaluationResponse {
  permit: PermitToken | null;
  failedGuard: string | null;
  reason: string | null;
  traceId: UUID;
}

export type GuardName =
  | 'G0_AUTH_VALID'
  | 'G1_OAUTH_VALID'
  | 'G2_QUOTA_OK'
  | 'G3_RISK_TIER_OK'
  | 'G4_KB_GROUNDED'
  | 'G5_POLICY_OK'
  | 'G6_HUMAN_APPROVED'
  | 'G7_AUDIT_LOGGED'
  | 'G8_INTEGRATION_HEALTHY';

export interface GuardOutcome {
  name: GuardName;
  passed: boolean;
  detail: string;
  durationMs: number;
}
