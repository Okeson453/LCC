/**
 * Opportunity API — pipeline, discover, draft, apply, propose.
 *
 * Canonical paths from `config.ts::API_PATHS.opportunities`.
 */

import { apiFetch } from './client';
import { API_PATHS } from './config';
import type {
  ApplicationDraft,
  JobRef,
  Opportunity,
  OpportunityDetail,
  ProposalDraft,
} from '@lcc/api-types';

export async function listOpportunities(_memberId: string): Promise<Opportunity[]> {
  return apiFetch<Opportunity[]>(API_PATHS.opportunities.list);
}

export async function discoverOpportunities(_memberId: string): Promise<JobRef> {
  return apiFetch<JobRef>('/api/v1/opportunities/discover', { method: 'POST' });
}

export async function getOpportunity(
  _memberId: string,
  opportunityId: string,
): Promise<OpportunityDetail> {
  // Detail endpoint follows `lcc-api-canonical.yaml::OpportunityDetail`.
  return apiFetch<OpportunityDetail>(`/api/v1/opportunities/${opportunityId}`);
}

export async function draftApplication(
  _memberId: string,
  opportunityId: string,
): Promise<ApplicationDraft> {
  return apiFetch<ApplicationDraft>(API_PATHS.opportunities.apply(opportunityId), {
    method: 'PATCH',
  });
}

export async function draftProposal(
  _memberId: string,
  opportunityId: string,
): Promise<ProposalDraft> {
  return apiFetch<ProposalDraft>('/api/v1/opportunities/proposals/draft', {
    method: 'POST',
    body: { opportunity_id: opportunityId },
  });
}
