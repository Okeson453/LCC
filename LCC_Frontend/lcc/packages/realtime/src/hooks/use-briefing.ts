/**
 * useBriefing — typed convenience for ws/briefing.
 */

import { useChannel } from './use-channel';
import type { RealtimeClient } from '../client';
import type { BriefingRefreshPayload } from '../channels/briefing';

export function useBriefing(client: RealtimeClient) {
  return useChannel<BriefingRefreshPayload>(client, 'briefing.refresh', { autoConnect: true });
}
export { useBriefing as useBriefingChannel };
