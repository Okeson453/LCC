import { useMutation, type UseMutationResult } from '@tanstack/react-query';
import { runQualityCheck } from '../content';
import type { QualityReport } from '@lcc/api-types';

export function useQualityCheck(
  memberId: string,
  contentId: string,
): UseMutationResult<QualityReport, Error, void> {
  return useMutation<QualityReport, Error, void>({
    mutationFn: () => runQualityCheck(memberId, contentId),
  });
}
