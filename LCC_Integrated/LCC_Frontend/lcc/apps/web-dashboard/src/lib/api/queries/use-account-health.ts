import { useQuery, type UseQueryResult } from '@tanstack/react-query';
import { fetchAccountHealth } from '../analytics';
import type { AccountHealth } from '@lcc/api-types';

export function useAccountHealth(
  memberId: string | undefined,
): UseQueryResult<AccountHealth, Error> {
  return useQuery({
    queryKey: ['analytics', 'account-health', memberId] as const,
    queryFn: () => fetchAccountHealth(memberId!),
    enabled: Boolean(memberId),
    refetchInterval: 60_000,
  });
}
