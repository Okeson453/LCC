import { useQuery, type UseQueryResult } from '@tanstack/react-query';
import { listSequences, getSequence } from '../outreach';
import type { Sequence, SequenceDetail } from '@lcc/api-types';

export function useSequences(
  memberId: string | undefined,
): UseQueryResult<Sequence[], Error> {
  return useQuery({
    queryKey: ['outreach', 'sequences', memberId] as const,
    queryFn: () => listSequences(memberId!),
    enabled: Boolean(memberId),
  });
}

export function useSequence(
  memberId: string | undefined,
  sequenceId: string | undefined,
): UseQueryResult<SequenceDetail, Error> {
  return useQuery({
    queryKey: ['outreach', 'sequence', memberId, sequenceId] as const,
    queryFn: () => getSequence(memberId!, sequenceId!),
    enabled: Boolean(memberId && sequenceId),
  });
}
