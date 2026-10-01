import { useQuery, type UseQueryResult } from '@tanstack/react-query';
import { listComplianceConfigVersions } from '../admin';
import type { ComplianceConfigVersion } from '@lcc/api-types';

export function useComplianceConfigVersions(): UseQueryResult<ComplianceConfigVersion[], Error> {
  return useQuery({
    queryKey: ['admin', 'compliance', 'config-versions'] as const,
    queryFn: () => listComplianceConfigVersions(),
  });
}

export function useActiveComplianceConfig(): UseQueryResult<ComplianceConfigVersion | null, Error> {
  const q = useComplianceConfigVersions();
  return {
    ...q,
    data: q.data?.find((v) => v.status === 'active') ?? null,
  } as UseQueryResult<ComplianceConfigVersion | null, Error>;
}
