import { useMutation, type UseMutationResult } from '@tanstack/react-query';
import { composeContent } from '../content';
import type { ComposeRequest, ContentVariant } from '@lcc/api-types';

export function useComposeContent(memberId: string): UseMutationResult<ContentVariant[], Error, ComposeRequest> {
  return useMutation<ContentVariant[], Error, ComposeRequest>({
    mutationFn: (input) => composeContent(memberId, input),
  });
}
