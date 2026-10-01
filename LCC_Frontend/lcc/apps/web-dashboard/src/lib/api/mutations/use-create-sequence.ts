import { useMutation, useQueryClient, type UseMutationResult } from '@tanstack/react-query';
import { createSequence } from '../outreach';
import type { Sequence, SequenceCreate } from '@lcc/api-types';

export function useCreateSequence(memberId: string): UseMutationResult<Sequence, Error, SequenceCreate> {
  const queryClient = useQueryClient();
  return useMutation<Sequence, Error, SequenceCreate>({
    mutationFn: (input) => createSequence(memberId, input),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['outreach', 'sequences', memberId] });
    },
  });
}
