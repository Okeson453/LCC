/* eslint-disable */
/**
 * @generated from schemas/events/member.created.schema.json
 */

import type { UUID, DateTime } from './common';

export interface MemberCreatedEvent {
  event_id: UUID;
  event_type: 'member.created';
  occurred_at: DateTime;
  trace_id: UUID;
  member_id: UUID;
  email: string;
  display_name: string;
  goal_mode: 'job_hunting' | 'client_acquisition' | 'hybrid';
  timezone: string;
}
