import { useMutation, useQueryClient, type UseMutationResult } from '@tanstack/react-query';
import { activateComplianceConfig } from '../admin';
import type { ComplianceConfigVersion } from '@lcc/api-types';

export function useActivateComplianceConfig(): UseMutationResult<ComplianceConfigVersion, Error, string> {
  const queryClient = useQueryClient();
  return useMutation<ComplianceConfigVersion, Error, string>({
    mutationFn: (versionId) => activateComplianceConfig(versionId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['admin', 'compliance', 'config-versions'] });
    },
  });
}
