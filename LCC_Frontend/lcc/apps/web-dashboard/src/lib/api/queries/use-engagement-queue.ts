import { useQuery, type UseQueryResult } from '@tanstack/react-query';
import { fetchQueue } from '../engagement';
import type { QueueItem } from '@lcc/api-types';

export function useEngagementQueue(
  memberId: string | undefined,
): UseQueryResult<QueueItem[], Error> {
  return useQuery({
    queryKey: ['engagement', 'queue', memberId] as const,
    queryFn: () => fetchQueue(memberId!),
    enabled: Boolean(memberId),
    refetchInterval: 60_000,
  });
}
