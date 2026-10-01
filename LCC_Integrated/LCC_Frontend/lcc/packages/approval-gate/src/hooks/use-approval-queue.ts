/**
 * useApprovalQueue — TanStack Query wrapper for pending approvals.
 *
 * Apps must provide their own queryClient and apiFetch; this hook provides
 * the typed query key and consumer wiring.
 */

import { useQuery, type UseQueryResult } from '@tanstack/react-query';
import type { Approval } from '@lcc/api-types';

export interface ApprovalQueueFilter {
  status?: 'pending' | 'decided' | 'expired';
  tier?: 1 | 2 | 3 | 4 | 5;
}

export interface UseApprovalQueueOptions {
  filter?: ApprovalQueueFilter;
  /** Function to fetch approvals; provided by the host app. */
  fetcher: (filter: ApprovalQueueFilter) => Promise<Approval[]>;
  refetchInterval?: number | false;
  enabled?: boolean;
}

export const approvalQueryKeys = {
  all: ['approvals'] as const,
  list: (filter: ApprovalQueueFilter) => [...approvalQueryKeys.all, 'list', filter] as const,
  detail: (id: string) => [...approvalQueryKeys.all, 'detail', id] as const,
};

export function useApprovalQueue(
  options: UseApprovalQueueOptions,
): UseQueryResult<Approval[], Error> {
  return useQuery({
    queryKey: approvalQueryKeys.list(options.filter ?? {}),
    queryFn: () => options.fetcher(options.filter ?? {}),
    refetchInterval: options.refetchInterval ?? false,
    enabled: options.enabled ?? true,
  });
}
