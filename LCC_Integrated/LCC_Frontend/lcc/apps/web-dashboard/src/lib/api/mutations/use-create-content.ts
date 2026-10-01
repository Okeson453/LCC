import { useMutation, useQueryClient, type UseMutationResult } from '@tanstack/react-query';
import { createContent } from '../content';
import type { ContentCreate, ContentItem } from '@lcc/api-types';

export function useCreateContent(memberId: string): UseMutationResult<ContentItem, Error, ContentCreate> {
  const queryClient = useQueryClient();
  return useMutation<ContentItem, Error, ContentCreate>({
    mutationFn: (input) => createContent(memberId, input),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['content', 'list', memberId] });
    },
  });
}
