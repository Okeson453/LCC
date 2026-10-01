/**
 * Default MSW handlers — happy-path responses for every endpoint.
 *
 * Tests can override via `server.use(http.get('/...', ...))` per test.
 */

import { http, HttpResponse } from 'msw';
import { mockMember } from '../fixtures/member';
import {
  mockApprovalTier3,
  mockApprovalTier5,
  mockApprovalApproved,
} from '../fixtures/approval';
import { mockContentItemDraft, mockContentItemPublished } from '../fixtures/content-item';
import { mockContact } from '../fixtures/contact';
import { mockOpportunityJob } from '../fixtures/opportunity';

const API = '';

export const handlers = [
  // Member
  http.get(`${API}/members/me`, () => HttpResponse.json(mockMember)),
  http.patch(`${API}/members/me`, () => HttpResponse.json(mockMember)),
  http.get(`${API}/members/me/settings`, () =>
    HttpResponse.json({
      goal_mode: 'hybrid',
      notifications_enabled: true,
      quiet_hours_start: null,
      quiet_hours_end: null,
      theme: 'dark',
    }),
  ),

  // Briefing
  http.get(`${API}/members/:id/briefing/today`, () =>
    HttpResponse.json({
      date: new Date().toISOString().slice(0, 10),
      sections: [
        {
          kind: 'approvals_due',
          title: 'Approvals Due',
          items: [
            {
              id: mockApprovalTier3.id,
              kind: 'approval',
              title: 'Approve: Send connection',
              summary: 'Jane Doe — CTO at Acme',
              action_url: '/approvals/' + mockApprovalTier3.id,
              tier: 3,
            },
          ],
        },
      ],
      generated_at: new Date().toISOString(),
    }),
  ),

  // Approvals
  http.get(`${API}/members/:id/approvals`, () =>
    HttpResponse.json([mockApprovalTier3, mockApprovalTier5]),
  ),
  http.get(`${API}/members/:id/approvals/:approvalId`, ({ params }) => {
    if (params.approvalId === mockApprovalApproved.id) {
      return HttpResponse.json(mockApprovalApproved);
    }
    if (params.approvalId === mockApprovalTier5.id) {
      return HttpResponse.json(mockApprovalTier5);
    }
    return HttpResponse.json(mockApprovalTier3);
  }),
  http.post(`${API}/members/:id/approvals/:approvalId/decide`, () =>
    HttpResponse.json({
      approval: mockApprovalApproved,
      governance: { permit: true, failed_guard: null, reason: null },
    }),
  ),

  // Content
  http.get(`${API}/members/:id/content`, () =>
    HttpResponse.json([mockContentItemDraft, mockContentItemPublished]),
  ),
  http.post(`${API}/members/:id/content/compose`, () =>
    HttpResponse.json([
      {
        variant: 'authority',
        body: 'Authority variant body',
        kb_refs: [],
      },
      {
        variant: 'contrarian',
        body: 'Contrarian variant body',
        kb_refs: [],
      },
    ]),
  ),

  // Engagement
  http.get(`${API}/members/:id/engagement/queue`, () => HttpResponse.json([])),
  http.get(`${API}/members/:id/engagement/inbox`, () => HttpResponse.json([])),

  // Network
  http.get(`${API}/members/:id/contacts`, () => HttpResponse.json([mockContact])),

  // Outreach
  http.get(`${API}/members/:id/sequences`, () => HttpResponse.json([])),

  // Opportunities
  http.get(`${API}/members/:id/opportunities`, () =>
    HttpResponse.json([mockOpportunityJob]),
  ),

  // Analytics
  http.get(`${API}/members/:id/analytics/account-health`, () =>
    HttpResponse.json({
      score: 0.86,
      components: {
        quota_remaining: 0.75,
        grounding_score: 0.9,
        approval_throughput: 0.8,
        recent_denial_rate: 0.05,
      },
      quota_consumed_today: 5,
      quota_cap_today: 18,
      computed_at: new Date().toISOString(),
      is_restricted: false,
      restriction_reason: null,
    }),
  ),

  // Compliance
  http.get(`${API}/admin/compliance/config-versions`, () =>
    HttpResponse.json([
      {
        id: 'cfg-active',
        version: 7,
        status: 'active',
        config: {
          daily_action_cap: 18,
          connection_per_day_cap: 10,
          dm_per_day_cap: 8,
          min_grounding_score: 0.7,
          tier2_approval_required: true,
          tier3_approval_required: true,
        },
        reviewers: [],
        created_at: '2026-04-01T00:00:00Z',
        activated_at: '2026-04-01T01:00:00Z',
      },
    ]),
  ),
];
