/**
 * Event envelope — every WS/SSE/polling message uses this shape.
 */

import type { UUID, DateTime } from '@lcc/api-types';

export type RealtimeMessageType =
  | 'briefing.refresh'
  | 'approvals.refresh'
  | 'engagement.refresh'
  | 'compliance.restriction_changed'
  | 'sequence.updated'
  | 'integration.action_pushed'
  | 'ping'
  | 'pong'
  | 'error';

export interface EventEnvelope<TPayload = unknown> {
  event_id: UUID;
  event_type: RealtimeMessageType;
  occurred_at: DateTime;
  trace_id: UUID;
  member_id: UUID;
  payload: TPayload;
}

const envelopeShape = {
  event_id: 'uuid',
  event_type: 'string',
  occurred_at: 'date-time',
  trace_id: 'uuid',
  member_id: 'uuid',
  payload: 'unknown',
} as const;

export function parseEnvelope<TPayload = unknown>(raw: string): EventEnvelope<TPayload> {
  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch (e) {
    throw new Error(`Malformed envelope: ${(e as Error).message}`);
  }
  if (typeof parsed !== 'object' || parsed === null) {
    throw new Error('Envelope is not an object');
  }
  const obj = parsed as Record<string, unknown>;
  for (const key of Object.keys(envelopeShape)) {
    if (!(key in obj)) {
      throw new Error(`Envelope missing field: ${key}`);
    }
  }
  return parsed as EventEnvelope<TPayload>;
}

export function isPingFrame(raw: string): boolean {
  try {
    const obj = JSON.parse(raw);
    return obj && typeof obj === 'object' && obj.type === 'ping';
  } catch {
    return false;
  }
}

export function isPongFrame(raw: string): boolean {
  try {
    const obj = JSON.parse(raw);
    return obj && typeof obj === 'object' && obj.type === 'pong';
  } catch {
    return false;
  }
}
