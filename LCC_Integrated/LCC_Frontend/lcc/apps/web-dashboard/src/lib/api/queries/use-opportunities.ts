import { useQuery, type UseQueryResult } from '@tanstack/react-query';
import { listOpportunities, getOpportunity } from '../opportunity';
import type { Opportunity, OpportunityDetail } from '@lcc/api-types';

export function useOpportunities(
  memberId: string | undefined,
): UseQueryResult<Opportunity[], Error> {
  return useQuery({
    queryKey: ['opportunity', 'list', memberId] as const,
    queryFn: () => listOpportunities(memberId!),
    enabled: Boolean(memberId),
  });
}

export function useOpportunity(
  memberId: string | undefined,
  opportunityId: string | undefined,
): UseQueryResult<OpportunityDetail, Error> {
  return useQuery({
    queryKey: ['opportunity', 'detail', memberId, opportunityId] as const,
    queryFn: () => getOpportunity(memberId!, opportunityId!),
    enabled: Boolean(memberId && opportunityId),
  });
}
