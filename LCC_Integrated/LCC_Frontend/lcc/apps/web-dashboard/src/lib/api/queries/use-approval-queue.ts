import { useApprovalQueue as useApprovalQueueBase } from '@lcc/approval-gate';
import { listApprovals } from '../approval';
import type { RiskTier } from '@lcc/api-types';

export interface UseApprovalQueueArgs {
  memberId: string | undefined;
  status?: 'pending' | 'decided' | 'expired';
  tier?: RiskTier;
  refetchInterval?: number | false;
}

export function useApprovalQueue({ memberId, status, tier, refetchInterval }: UseApprovalQueueArgs) {
  return useApprovalQueueBase({
    filter: { status, tier },
    fetcher: (f) => listApprovals(memberId!, f),
    refetchInterval: refetchInterval ?? 30_000,
    enabled: Boolean(memberId),
  });
}
