import { useMutation, useQueryClient, type UseMutationResult } from '@tanstack/react-query';
import { scheduleContent } from '../content';
import type { ContentItem } from '@lcc/api-types';

export function useScheduleContent(
  memberId: string,
  contentId: string,
): UseMutationResult<ContentItem, Error, string> {
  const queryClient = useQueryClient();
  return useMutation<ContentItem, Error, string>({
    mutationFn: (scheduledAt) => scheduleContent(memberId, contentId, scheduledAt),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['content'] });
    },
  });
}
