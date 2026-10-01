import { useQuery, type UseQueryResult } from '@tanstack/react-query';
import { getContentItem } from '../content';
import type { ContentItem } from '@lcc/api-types';

export function useContentItem(
  memberId: string | undefined,
  contentId: string | undefined,
): UseQueryResult<ContentItem, Error> {
  return useQuery({
    queryKey: ['content', 'item', memberId, contentId] as const,
    queryFn: () => getContentItem(memberId!, contentId!),
    enabled: Boolean(memberId && contentId),
  });
}
