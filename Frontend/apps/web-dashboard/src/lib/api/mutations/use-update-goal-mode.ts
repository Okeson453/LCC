import { useMutation, useQueryClient, type UseMutationResult } from '@tanstack/react-query';
import { updateMemberSettings } from '../members';
import type { MemberSettings, MemberSettingsUpdate } from '@lcc/api-types';

export function useUpdateGoalMode(): UseMutationResult<MemberSettings, Error, MemberSettingsUpdate> {
  const queryClient = useQueryClient();
  return useMutation<MemberSettings, Error, MemberSettingsUpdate>({
    mutationFn: (input) => updateMemberSettings(input),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['members', 'me'] });
    },
  });
}
