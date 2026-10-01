import { useQuery, type UseQueryResult } from '@tanstack/react-query';
import { listKbRecords } from '../kb';
import type { KbCategory, KbRecord } from '@lcc/api-types';

export function useKbRecords(
  memberId: string | undefined,
  category?: KbCategory,
): UseQueryResult<KbRecord[], Error> {
  return useQuery({
    queryKey: ['kb', 'records', memberId, category] as const,
    queryFn: () => listKbRecords(memberId!, category),
    enabled: Boolean(memberId),
  });
}
