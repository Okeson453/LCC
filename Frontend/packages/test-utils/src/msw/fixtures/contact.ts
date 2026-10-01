import type { Contact } from '@lcc/api-types';

export const mockContact: Contact = {
  id: 'p1d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f',
  member_id: '0d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f',
  display_name: 'Jane Doe',
  avatar_url: null,
  headline: 'CTO at Acme',
  company: { id: 'co-acme', name: 'Acme' },
  stage: 'engaged',
  tags: ['fintech', 'series-b'],
  last_interaction_at: new Date(Date.now() - 14 * 24 * 60 * 60 * 1000).toISOString(),
  warmth_score: 0.62,
  created_at: '2026-01-01T00:00:00Z',
};
