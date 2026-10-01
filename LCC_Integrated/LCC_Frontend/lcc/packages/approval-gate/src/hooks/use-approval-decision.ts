/**
 * useApprovalDecision — TanStack Query mutation hook for approve/reject.
 *
 * The host app provides the fetcher (which calls
 * `POST /members/{id}/approvals/{approvalId}/decide`). The mutation handles
 * cache invalidation: after a decision, the queue refetches and any
 * ApprovalGate components that subscribed to that approval id refetch too.
 */

import { useMutation, useQueryClient, type UseMutationResult } from '@tanstack/react-query';
import type { Approval } from '@lcc/api-types';
import { approvalQueryKeys } from './use-approval-queue';

export interface ApprovalDecisionInput {
  approvalId: string;
  decision: 'approve' | 'reject';
  editedPayload?: Record<string, unknown>;
  comment?: string;
}

export interface ApprovalDecisionOutput {
  approval: Approval;
  governance: {
    permit: boolean;
    failedGuard: string | null;
    reason: string | null;
  };
}

export interface UseApprovalDecisionOptions {
  /** Function to call the decide endpoint. */
  decider: (input: ApprovalDecisionInput) => Promise<ApprovalDecisionOutput>;
  /** Called on success — useful for toasts. */
  onSuccess?: (output: ApprovalDecisionOutput, input: ApprovalDecisionInput) => void;
  /** Called on error — useful for Sentry/telemetry. */
  onError?: (error: Error, input: ApprovalDecisionInput) => void;
}

export function useApprovalDecision(
  options: UseApprovalDecisionOptions,
): UseMutationResult<ApprovalDecisionOutput, Error, ApprovalDecisionInput> {
  const queryClient = useQueryClient();

  return useMutation<ApprovalDecisionOutput, Error, ApprovalDecisionInput>({
    mutationFn: (input) => options.decider(input),
    onSuccess: (output, input) => {
      // Invalidate the queue (the decided item leaves or transitions status)
      queryClient.invalidateQueries({ queryKey: approvalQueryKeys.all });
      // Update the single-approval cache
      queryClient.setQueryData(approvalQueryKeys.detail(input.approvalId), output.approval);
      options.onSuccess?.(output, input);
    },
    onError: (error, input) => {
      options.onError?.(error, input);
    },
  });
}
