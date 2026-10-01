/**
 * Channel subscription: ws/compliance
 *
 * Pushes restriction state changes (entered restricted, cleared restricted).
 * The frontend MUST respond by enabling/disabling action surfaces globally.
 */

import type { EventEnvelope } from '../envelope';
import type { RealtimeClient } from '../client';

export interface ComplianceEventPayload {
  memberId: string;
  isRestricted: boolean;
  reason: 'denial_rate' | 'manual_review' | 'oauth_expired' | 'governance_fail' | 'abuse_signal' | 'none';
  reasonDetail: string | null;
  triggeredAt: string | null;
}

export function subscribeComplianceChannel(
  client: RealtimeClient,
  listener: (env: EventEnvelope<ComplianceEventPayload>) => void,
): () => void {
  return client.subscribeByType('compliance.restriction_changed', listener as never);
}

export const COMPLIANCE_CHANNEL_PATH = '/api/v1/ws/compliance';
