import type { OpportunityActionPlan } from '@lcc/api-types';

export interface PipelineCardViewModel {
  id: string;
  title: string;
  company: string | null;
  fitScore: number;
  status: string;
  actionPlan: OpportunityActionPlan;
  capturedAt: string;
}
