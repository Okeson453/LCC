/**
 * useReject - wired approval-decision mutation pre-configured with decision: 'reject'.
 * Rejections bypass Tier UX rules and go straight to the backend decision endpoint.
 */

import { useApprovalDecision, type ApprovalDecisionInput } from '@lcc/approval-gate';
import { decideApproval } from '../approval';

export function useReject(memberId: string) {
  if (!memberId) {
    throw new Error('[useReject] memberId is required');
  }
  return useApprovalDecision({
    decider: (input: ApprovalDecisionInput) => {
      const decision = {
        decision: 'reject',
        comment: input.comment,
      };
      return decideApproval(memberId, input.approvalId, decision);
    },
  });
}

