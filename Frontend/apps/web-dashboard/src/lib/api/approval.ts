/**
 * Approvals API — list, detail, decide.
 *
 * Canonical paths from `config.ts::API_PATHS.approvals`.
 */

import { apiFetch } from './client';
import { API_PATHS } from './config';
import type { Approval, ApprovalDecision } from '@lcc/api-types';
import type { ApprovalDecisionOutput } from '@lcc/approval-gate';

export async function listApprovals(
  _memberId: string,
  filter?: { status?: 'pending' | 'decided' | 'expired'; tier?: 1 | 2 | 3 | 4 | 5 },
): Promise<Approval[]> {
  return apiFetch<Approval[]>(API_PATHS.approvals.list, { query: filter });
}

export async function getApproval(_memberId: string, approvalId: string): Promise<Approval> {
  return apiFetch<Approval>(API_PATHS.approvals.item(approvalId));
}

/** Raw wire response of the decide endpoint (snake_case, as the API returns it). */
export interface DecideApprovalWireResponse {
  approval: Approval;
  governance: { permit: boolean; failed_guard: string | null; reason: string | null };
}

/**
 * Decide an approval and return the presentation shape `useApprovalDecision`
 * expects.
 *
 * The endpoint answers with `governance.failed_guard`; the hook's
 * `ApprovalDecisionOutput` declares `failedGuard`. Passing the wire response
 * straight through failed to typecheck at every call site, so the mapping lives
 * here, at the single boundary between the two.
 */
export async function decideApproval(
  memberId: string,
  approvalId: string,
  decision: ApprovalDecision,
): Promise<ApprovalDecisionOutput> {
  const res = await postDecide(memberId, approvalId, decision);
  return {
    approval: res.approval,
    governance: {
      permit: res.governance.permit,
      failedGuard: res.governance.failed_guard,
      reason: res.governance.reason,
    },
  };
}

async function postDecide(
  _memberId: string,
  approvalId: string,
  decision: ApprovalDecision,
): Promise<DecideApprovalWireResponse> {
  // `ApprovalDecision` carries no `version` field (generated/http/approval.ts),
  // so sending `version: decision.version ?? 0` referenced a property that does
  // not exist. The optimistic-concurrency version, when the endpoint requires
  // one, is carried on `Approval`; send the body as declared.
  return apiFetch(API_PATHS.approvals.decide(approvalId), {
    method: 'PATCH',
    body: decision,
  });
}
