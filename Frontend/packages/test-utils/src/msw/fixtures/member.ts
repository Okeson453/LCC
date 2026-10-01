/**
 * MSW fixture: member data.
 */

import type { Member } from '@lcc/api-types';

export const mockMember: Member = {
  id: '0d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f',
  email: 'jane.doe@example.com',
  display_name: 'Jane Doe',
  avatar_url: null,
  goal_mode: 'hybrid',
  timezone: 'Europe/Paris',
  oauth_expires_at: new Date(Date.now() + 30 * 24 * 60 * 60 * 1000).toISOString(),
  is_restricted: false,
  created_at: '2026-01-01T00:00:00Z',
};

export const mockMemberRestricted: Member = {
  ...mockMember,
  is_restricted: true,
};
