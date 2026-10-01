/**
 * Channel subscription: ws/approvals
 *
 * Pushes pending-approval queue updates. Each envelope represents a single
 * approval that has been created, updated, decided, or expired.
 */

import type { EventEnvelope } from '../envelope';
import type { RealtimeClient } from '../client';

export interface ApprovalEventPayload {
  approvalId: string;
  /** Action kind that mutated the approval. */
  action: 'created' | 'updated' | 'decided' | 'expired';
  status: 'pending' | 'approved' | 'rejected' | 'expired' | 'failed';
  /** Trace ID of the triggering action. */
  traceId: string;
}

export function subscribeApprovalsChannel(
  client: RealtimeClient,
  listener: (env: EventEnvelope<ApprovalEventPayload>) => void,
): () => void {
  return client.subscribeByType('approvals.refresh', listener as never);
}

export const APPROVALS_CHANNEL_PATH = '/api/v1/ws/approvals';
