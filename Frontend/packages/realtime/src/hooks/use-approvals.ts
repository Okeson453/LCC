/**
 * useApprovals — typed convenience for ws/approvals.
 */

import { useChannel } from './use-channel';
import type { RealtimeClient } from '../client';
import type { ApprovalEventPayload } from '../channels/approvals';

export function useApprovals(client: RealtimeClient) {
  return useChannel<ApprovalEventPayload>(client, 'approvals.refresh', { autoConnect: true });
}
