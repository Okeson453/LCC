/**
 * MSW fixture: approval data.
 */

import type { Approval } from '@lcc/api-types';

export const mockApprovalTier3: Approval = {
  id: 'a1d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f',
  member_id: '0d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f',
  action_type: 'send_connection',
  tier: 3,
  status: 'pending',
  payload: {
    person_id: 'p-jane-doe',
    person_name: 'Jane Doe',
    message: 'I would love to connect!',
  },
  kb_refs: [
    {
      record_id: 'r-1',
      title: 'Senior PM resume',
      category: 'resume',
      excerpt: 'Senior Product Manager with 8+ years…',
      url: null,
    },
    {
      record_id: 'r-2',
      title: 'ICP — Series B SaaS PMs',
      category: 'ideal_customer',
      excerpt: 'Target: PMs at Series B SaaS companies…',
      url: null,
    },
  ],
  governance: null,
  trace_id: 't1d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f',
  idempotency_key: 'idem-1234567890abcdef',
  created_at: new Date().toISOString(),
  decided_at: null,
};

export const mockApprovalTier5: Approval = {
  ...mockApprovalTier3,
  id: 'a2d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f',
  action_type: 'apply_opportunity',
  tier: 5,
  payload: {
    opportunity_id: 'o-acme-cto',
    opportunity_title: 'CTO at Acme',
    body: 'I am writing to apply for…',
  },
};

export const mockApprovalApproved: Approval = {
  ...mockApprovalTier3,
  status: 'approved',
  decided_at: new Date().toISOString(),
};
