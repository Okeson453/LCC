/* eslint-disable */
/**
 * @generated from schemas/events/compliance.restriction_detected.schema.json
 */

import type { UUID, DateTime } from './common';

export interface ComplianceRestrictionDetectedEvent {
  event_id: UUID;
  event_type: 'compliance.restriction_detected';
  occurred_at: DateTime;
  trace_id: UUID;
  member_id: UUID;
  reason: 'denial_rate' | 'manual_review' | 'oauth_expired' | 'governance_fail' | 'abuse_signal';
  reason_detail: string;
}
