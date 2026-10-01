/* eslint-disable */
/**
 * @generated from schemas/events/content.item.approved.schema.json
 */

import type { UUID, DateTime } from './common';

export interface ContentItemApprovedEvent {
  event_id: UUID;
  event_type: 'content.item.approved';
  occurred_at: DateTime;
  trace_id: UUID;
  member_id: UUID;
  content_item_id: UUID;
  approval_id: UUID;
}
