import { describe, expect, it } from 'vitest';
import {
  memberSchema,
  approvalSchema,
  briefingSchema,
  accountHealthSchema,
  restrictionStateSchema,
  validateApiResponse,
  ApiContractError,
} from '../src/runtime/zod-schemas';

describe('zod-schemas', () => {
  describe('memberSchema', () => {
    it('parses a valid member', () => {
      const data = {
        id: '0d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f',
        email: 'user@example.com',
        display_name: 'Jane Doe',
        avatar_url: null,
        goal_mode: 'hybrid',
        timezone: 'Europe/Paris',
        oauth_expires_at: '2026-05-01T00:00:00Z',
        is_restricted: false,
        created_at: '2026-01-01T00:00:00Z',
      };
      const result = memberSchema.safeParse(data);
      expect(result.success).toBe(true);
    });

    it('rejects an invalid email', () => {
      const data = {
        id: '0d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f',
        email: 'not-an-email',
        display_name: 'Jane',
        avatar_url: null,
        goal_mode: 'hybrid',
        timezone: 'Europe/Paris',
        oauth_expires_at: '2026-05-01T00:00:00Z',
        is_restricted: false,
        created_at: '2026-01-01T00:00:00Z',
      };
      const result = memberSchema.safeParse(data);
      expect(result.success).toBe(false);
    });
  });

  describe('approvalSchema', () => {
    it('parses a Tier 3 approval', () => {
      const data = {
        id: '0d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f',
        member_id: '1d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f',
        action_type: 'send_connection',
        tier: 3,
        status: 'pending',
        payload: {},
        kb_refs: [
          {
            record_id: '2d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f',
            title: 'About me',
            category: 'resume',
            excerpt: 'Senior PM...',
            url: null,
          },
        ],
        governance: null,
        trace_id: '3d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f',
        idempotency_key: 'idem-1234567890abcdef',
        created_at: '2026-01-01T00:00:00Z',
        decided_at: null,
      };
      const result = approvalSchema.safeParse(data);
      expect(result.success).toBe(true);
    });

    it('rejects invalid tier (out of range)', () => {
      const data = {
        id: '0d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f',
        member_id: '1d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f',
        action_type: 'send_connection',
        tier: 6,
        status: 'pending',
        payload: {},
        kb_refs: [],
        governance: null,
        trace_id: '3d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f',
        idempotency_key: 'idem-1234567890abcdef',
        created_at: '2026-01-01T00:00:00Z',
        decided_at: null,
      };
      const result = approvalSchema.safeParse(data);
      expect(result.success).toBe(false);
    });
  });

  describe('briefingSchema', () => {
    it('parses a briefing with multiple sections', () => {
      const data = {
        date: '2026-04-22',
        sections: [
          {
            kind: 'approvals_due',
            title: 'Approvals Due',
            items: [
              {
                id: '0d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f',
                kind: 'approval',
                title: 'Approve: Send connection',
                summary: 'Jane Doe — CTO at Acme',
                action_url: '/approvals/0d9b2a5b',
                tier: 3,
              },
            ],
          },
          {
            kind: 'engagement',
            title: 'Engagement',
            items: [],
          },
        ],
        generated_at: '2026-04-22T07:00:00Z',
      };
      const result = briefingSchema.safeParse(data);
      expect(result.success).toBe(true);
    });
  });

  describe('restrictionStateSchema', () => {
    it('parses an unrestricted state', () => {
      const data = {
        member_id: '0d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f',
        is_restricted: false,
        reason: 'none',
        reason_detail: null,
        triggered_at: null,
        cleared_at: null,
      };
      const result = restrictionStateSchema.safeParse(data);
      expect(result.success).toBe(true);
    });

    it('parses a manual-review restriction', () => {
      const data = {
        member_id: '0d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f',
        is_restricted: true,
        reason: 'manual_review',
        reason_detail: 'Operator Jane placed under review after complaint',
        triggered_at: '2026-04-22T07:00:00Z',
        cleared_at: null,
      };
      const result = restrictionStateSchema.safeParse(data);
      expect(result.success).toBe(true);
    });
  });

  describe('validateApiResponse', () => {
    it('returns parsed data on success', () => {
      const data = {
        member_id: '0d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f',
        is_restricted: false,
        reason: 'none',
        reason_detail: null,
        triggered_at: null,
        cleared_at: null,
      };
      const result = validateApiResponse(restrictionStateSchema, data, {
        url: '/x',
        status: 200,
        traceId: '0d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f',
      });
      expect(result.is_restricted).toBe(false);
    });

    it('throws ApiContractError on mismatch', () => {
      const data = { invalid: 'shape' };
      expect(() =>
        validateApiResponse(accountHealthSchema, data, {
          url: '/x',
          status: 200,
          traceId: '0d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f',
        }),
      ).toThrow(ApiContractError);
    });
  });
});
