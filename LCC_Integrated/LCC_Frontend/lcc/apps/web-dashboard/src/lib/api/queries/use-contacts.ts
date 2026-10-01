import { useQuery, type UseQueryResult } from '@tanstack/react-query';
import { listContacts } from '../network';
import type { Contact, RelationshipStage } from '@lcc/api-types';

export function useContacts(
  memberId: string | undefined,
  filter?: { stage?: RelationshipStage; expand?: 'company' },
): UseQueryResult<Contact[], Error> {
  return useQuery({
    queryKey: ['network', 'contacts', memberId, filter] as const,
    queryFn: () => listContacts(memberId!, filter),
    enabled: Boolean(memberId),
    staleTime: 60_000,
  });
}
