import { useQuery, type UseQueryResult } from '@tanstack/react-query';
import { listContent } from '../content';
import type { ContentItem, ContentStatus } from '@lcc/api-types';

export function useContentList(
  memberId: string | undefined,
  status?: ContentStatus,
): UseQueryResult<ContentItem[], Error> {
  return useQuery({
    queryKey: ['content', 'list', memberId, status] as const,
    queryFn: () => listContent(memberId!, status),
    enabled: Boolean(memberId),
  });
}
