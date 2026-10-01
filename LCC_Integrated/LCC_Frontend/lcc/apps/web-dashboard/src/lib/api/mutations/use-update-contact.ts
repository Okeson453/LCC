import { useMutation, useQueryClient, type UseMutationResult } from '@tanstack/react-query';
import { updateContact } from '../network';
import type { Contact, ContactUpdate } from '@lcc/api-types';

export function useUpdateContact(
  memberId: string,
  contactId: string,
): UseMutationResult<Contact, Error, ContactUpdate> {
  const queryClient = useQueryClient();
  return useMutation<Contact, Error, ContactUpdate>({
    mutationFn: (input) => updateContact(memberId, contactId, input),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['network'] });
    },
  });
}
