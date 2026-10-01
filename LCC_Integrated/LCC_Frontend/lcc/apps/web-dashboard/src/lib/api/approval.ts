/**
 * Approvals API — list, detail, decide.
 *
 * Canonical paths from `config.ts::API_PATHS.approvals`.
 */

import { apiFetch } from './client';
import { API_PATHS } from './config';
import type { Approval, ApprovalDecision } from '@lcc/api-types';

export async function listApprovals(
  _memberId: string,
  filter?: { status?: 'pending' | 'decided' | 'expired'; tier?: 1 | 2 | 3 | 4 | 5 },
): Promise<Approval[]> {
  return apiFetch<Approval[]>(API_PATHS.approvals.list, { query: filter });
}

export async function getApproval(_memberId: string, approvalId: string): Promise<Approval> {
  return apiFetch<Approval>(API_PATHS.approvals.item(approvalId));
}

export async function decideApproval(
  _memberId: string,
  approvalId: string,
  decision: ApprovalDecision,
): Promise<{
  approval: Approval;
  governance: { permit: boolean; failed_guard: string | null; reason: string | null };
}> {
  return apiFetch(API_PATHS.approvals.decide(approvalId), {
    method: 'PATCH',
    body: { ...decision, version: decision.version ?? 0 },
  });
}
