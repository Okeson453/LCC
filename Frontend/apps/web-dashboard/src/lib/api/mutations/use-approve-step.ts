import { useMutation, useQueryClient, type UseMutationResult } from '@tanstack/react-query';
import { approveStep } from '../outreach';
import type { SequenceStep, StepApprovalRequest } from '@lcc/api-types';

export function useApproveStep(
  memberId: string,
  sequenceId: string,
): UseMutationResult<SequenceStep, Error, { stepId: string; request: StepApprovalRequest }> {
  const queryClient = useQueryClient();
  return useMutation<SequenceStep, Error, { stepId: string; request: StepApprovalRequest }>({
    mutationFn: ({ stepId, request }) => approveStep(memberId, sequenceId, stepId, request),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['outreach', 'sequence', memberId, sequenceId] });
    },
  });
}
