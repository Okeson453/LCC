/**
 * useCompliance — typed convenience for ws/compliance.
 *
 * CRITICAL: any `is_restricted=true` payload should immediately disable
 * all action surfaces globally. The `@lcc/compliance-state` package wraps
 * this hook to provide the banner + gate logic.
 */

import { useChannel } from './use-channel';
import type { RealtimeClient } from '../client';
import type { ComplianceEventPayload } from '../channels/compliance';

export function useCompliance(client: RealtimeClient) {
  return useChannel<ComplianceEventPayload>(client, 'compliance.restriction_changed', {
    autoConnect: true,
  });
}
export { useCompliance as useComplianceChannel };
