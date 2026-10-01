/**
 * useProfile — TanStack Query wrapper for the latest profile snapshot.
 */

import { useQuery, type UseQueryResult } from '@tanstack/react-query';
import { fetchLatestSnapshot } from '../profile';
import type { ProfileSnapshot } from '@lcc/api-types';

export function useProfile(memberId: string | undefined): UseQueryResult<ProfileSnapshot, Error> {
  return useQuery({
    queryKey: ['profile', 'snapshot', memberId] as const,
    queryFn: () => fetchLatestSnapshot(memberId!),
    enabled: Boolean(memberId),
    staleTime: 60_000,
  });
}
