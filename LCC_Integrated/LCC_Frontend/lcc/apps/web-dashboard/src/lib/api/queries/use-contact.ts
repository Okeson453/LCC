import { useQuery, type UseQueryResult } from '@tanstack/react-query';
import { getContact } from '../network';
import type { Contact } from '@lcc/api-types';

export function useContact(
  memberId: string | undefined,
  contactId: string | undefined,
): UseQueryResult<Contact, Error> {
  return useQuery({
    queryKey: ['network', 'contact', memberId, contactId] as const,
    queryFn: () => getContact(memberId!, contactId!),
    enabled: Boolean(memberId && contactId),
  });
}
