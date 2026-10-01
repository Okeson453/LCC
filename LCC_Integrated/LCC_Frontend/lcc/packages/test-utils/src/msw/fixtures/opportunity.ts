import type { Opportunity } from '@lcc/api-types';

export const mockOpportunityJob: Opportunity = {
  id: 'o1d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f',
  member_id: '0d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f',
  title: 'CTO at Acme',
  kind: 'job',
  company: 'Acme',
  url: 'https://acme.example/jobs/cto',
  status: 'qualified',
  fit_score: 0.87,
  fit_breakdown: {
    skills: 0.9,
    seniority: 0.85,
    domain: 0.8,
    location: 0.95,
    comp: 0.7,
    culture: 0.85,
  },
  action_plan: 'apply_now',
  captured_at: new Date().toISOString(),
};
