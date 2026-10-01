/**
 * useApprovalDecisions — bulk mutation hook.
 *
 * Used by `<ApprovalQueue>` / `<ApprovalBulkBar>` for Tier 1–2 bulk approve.
 * Tier 3+ items MUST be reviewed individually and are excluded server-side.
 */

import { useMutation, useQueryClient, type UseMutationResult } from '@tanstack/react-query';
import type { Approval } from '@lcc/api-types';
import { approvalQueryKeys } from './use-approval-queue';

export interface BulkApprovalDecisionInput {
  approvalIds: string[];
  decision: 'approve' | 'reject';
  comment?: string;
}

export interface BulkApprovalDecisionOutput {
  results: Array<{
    approvalId: string;
    approval: Approval | null;
    error: string | null;
  }>;
}

export interface UseApprovalDecisionsOptions {
  decider: (input: BulkApprovalDecisionInput) => Promise<BulkApprovalDecisionOutput>;
  onSuccess?: (output: BulkApprovalDecisionOutput, input: BulkApprovalDecisionInput) => void;
  onError?: (error: Error, input: BulkApprovalDecisionInput) => void;
}

export function useApprovalDecisions(
  options: UseApprovalDecisionsOptions,
): UseMutationResult<BulkApprovalDecisionOutput, Error, BulkApprovalDecisionInput> {
  const queryClient = useQueryClient();

  return useMutation<BulkApprovalDecisionOutput, Error, BulkApprovalDecisionInput>({
    mutationFn: (input) => options.decider(input),
    onSuccess: (output, input) => {
      queryClient.invalidateQueries({ queryKey: approvalQueryKeys.all });
      options.onSuccess?.(output, input);
    },
    onError: (error, input) => options.onError?.(error, input),
  });
}
