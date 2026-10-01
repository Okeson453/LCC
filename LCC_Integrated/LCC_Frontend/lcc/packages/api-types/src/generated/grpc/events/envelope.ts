/* eslint-disable */
/**
 * @generated
 * Event envelope — every event on the WS/SSE bus uses this shape.
 */

import type { UUID, DateTime } from '../compliance/_common';

export interface EventEnvelope<TPayload = unknown> {
  eventId: UUID;
  eventType: string;
  occurredAt: DateTime;
  traceId: UUID;
  memberId: UUID;
  payload: TPayload;
}
