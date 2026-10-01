import type { KbCitation, RiskTier } from '@lcc/api-types';

export interface ApprovalQueueItemViewModel {
  id: string;
  actionType: string;
  tier: RiskTier;
  targetLabel: string;
  preview: string;
  kbRefCount: number;
  createdAt: string;
  kbRefs: KbCitation[];
}
