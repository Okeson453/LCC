/**
 * useApprove - wired approval-decision mutation (approve path).
 * Non-Negotiable: member ID is always supplied; empty/member-less decisions are rejected here.
 */

import { useApprovalDecision, type ApprovalDecisionInput } from '@lcc/approval-gate';
import { decideApproval } from '../approval';

export function useApprove(memberId: string) {
  if (!memberId) {
    throw new Error('[useApprove] memberId is required');
  }
  return useApprovalDecision({
    decider: (input: ApprovalDecisionInput) => {
      const decision = {
        decision: input.decision,
        edited_payload: input.editedPayload ?? null,
        comment: input.comment,
      };
      return decideApproval(memberId, input.approvalId, decision);
    },
  });
}

