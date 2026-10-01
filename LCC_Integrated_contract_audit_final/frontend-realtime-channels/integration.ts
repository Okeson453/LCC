/**
 * Channel subscription: ws/integration (Track B)
 *
 * Pushes pre-filled Track B actions from the Integration Gateway to the
 * Browser Extension. The extension listens for `integration.action_pushed`
 * events and renders the `<ActionPreview>` for the user to confirm.
 *
 * NEVER use this channel from the web dashboard — it is extension-only.
 */

import type { EventEnvelope } from '../envelope';
import type { RealtimeClient } from '../client';

export interface IntegrationActionPushedPayload {
  /** Unique action ID. */
  actionId: string;
  actionType:
    | 'profile_edit'
    | 'connection_request'
    | 'direct_message'
    | 'comment'
    | 'like';
  /** Pre-filled body / value the user will see in LinkedIn's DOM. */
  prefill: string;
  /** Target (person id, post id, profile field). */
  target: {
    kind: 'person' | 'post' | 'profile_field';
    id: string;
    displayName?: string;
  };
  /** KB citations grounding this action. */
  kbRefs: Array<{
    recordId: string;
    title: string;
    category: string;
    excerpt: string;
  }>;
  /** Time-bounded (action expires if not confirmed). */
  expiresAt: string;
  /** Backend trace_id for audit. */
  traceId: string;
}

export function subscribeIntegrationChannel(
  client: RealtimeClient,
  listener: (env: EventEnvelope<IntegrationActionPushedPayload>) => void,
): () => void {
  return client.subscribeByType('integration.action_pushed', listener as never);
}

export const INTEGRATION_CHANNEL_PATH = '/api/v1/ws/integration';
