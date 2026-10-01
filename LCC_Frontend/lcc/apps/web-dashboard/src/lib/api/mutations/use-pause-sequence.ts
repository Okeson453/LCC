import { useMutation, useQueryClient, type UseMutationResult } from '@tanstack/react-query';
import { pauseSequence } from '../outreach';
import type { Sequence } from '@lcc/api-types';

export function usePauseSequence(
  memberId: string,
  sequenceId: string,
): UseMutationResult<Sequence, Error, void> {
  const queryClient = useQueryClient();
  return useMutation<Sequence, Error, void>({
    mutationFn: () => pauseSequence(memberId, sequenceId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['outreach'] });
    },
  });
}
