/**
 * useApproval — single-approval query (detail view).
 */

import { useQuery, type UseQueryResult } from '@tanstack/react-query';
import type { Approval } from '@lcc/api-types';
import { approvalQueryKeys } from './use-approval-queue';

export interface UseApprovalOptions {
  fetcher: (id: string) => Promise<Approval>;
  enabled?: boolean;
}

export function useApproval(
  id: string,
  options: UseApprovalOptions,
): UseQueryResult<Approval, Error> {
  return useQuery({
    queryKey: approvalQueryKeys.detail(id),
    queryFn: () => options.fetcher(id),
    enabled: options.enabled ?? true,
  });
}
