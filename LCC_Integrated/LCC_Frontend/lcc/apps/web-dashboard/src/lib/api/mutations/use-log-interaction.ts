import { useMutation, useQueryClient, type UseMutationResult } from '@tanstack/react-query';
import { logInteraction } from '../network';
import type { Interaction, InteractionCreate } from '@lcc/api-types';

export function useLogInteraction(
  memberId: string,
  contactId: string,
): UseMutationResult<Interaction, Error, InteractionCreate> {
  const queryClient = useQueryClient();
  return useMutation<Interaction, Error, InteractionCreate>({
    mutationFn: (input) => logInteraction(memberId, contactId, input),
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: ['network', 'contact', memberId, contactId],
      });
    },
  });
}
