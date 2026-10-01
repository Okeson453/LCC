import { useMutation, useQueryClient, type UseMutationResult } from '@tanstack/react-query';
import { updateKbRecord, createKbRecord } from '../kb';
import type { KbRecord, KbRecordCreate, KbRecordUpdate } from '@lcc/api-types';

export function useUpdateKb(
  memberId: string,
): UseMutationResult<KbRecord, Error, { recordId?: string; input: KbRecordUpdate | KbRecordCreate }> {
  const queryClient = useQueryClient();
  return useMutation<KbRecord, Error, { recordId?: string; input: KbRecordUpdate | KbRecordCreate }>({
    mutationFn: ({ recordId, input }) => {
      if (recordId && 'category' in input) {
        return updateKbRecord(memberId, recordId, input as KbRecordUpdate);
      }
      if (recordId) {
        return updateKbRecord(memberId, recordId, input as KbRecordUpdate);
      }
      return createKbRecord(memberId, input as KbRecordCreate);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['kb', 'records', memberId] });
    },
  });
}
