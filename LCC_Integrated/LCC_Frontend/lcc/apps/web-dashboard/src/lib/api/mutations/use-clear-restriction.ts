import { useMutation, useQueryClient, type UseMutationResult } from '@tanstack/react-query';
import { clearRestriction } from '../admin';
import type { RestrictionState } from '@lcc/api-types';

export function useClearRestriction(): UseMutationResult<RestrictionState, Error, string> {
  const queryClient = useQueryClient();
  return useMutation<RestrictionState, Error, string>({
    mutationFn: (memberId) => clearRestriction(memberId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['admin', 'compliance'] });
    },
  });
}
