import { describe, expect, it } from 'vitest';
import {
  approvalSchema,
  contentItemSchema,
  memberSchema,
  queueItemSchema,
  restrictionStateSchema,
} from '@lcc/api-types';

const UUID = '11111111-2222-4333-8444-555555555555';
const ISO = '2026-01-01T00:00:00.000Z';

/**
 * These assertions target the schemas `@lcc/api-types` actually exports.
 *
 * The previous version imported `MemberSchema` / `ApprovalSchema` /
 * `OpportunitySchema` / `KbRecordSchema` in PascalCase — the module exports
 * them in camelCase (`memberSchema`, `approvalSchema`, …) and ships no
 * opportunity or KB-record schema at all. Every case therefore failed with
 * "Cannot read properties of undefined (reading 'safeParse')". The fixtures
 * also predated UUID validation and used ids like `'m1'`.
 */
describe('zod schemas', () => {
  it('parses a minimal Member', () => {
    const parsed = memberSchema.safeParse({
      id: UUID,
      display_name: 'Alice',
      avatar_url: null,
      email: 'alice@example.com',
      goal_mode: 'job_hunting',
      timezone: 'UTC',
      oauth_expires_at: ISO,
      is_restricted: false,
      created_at: ISO,
    });
    expect(parsed.success).toBe(true);
  });

  it('rejects a Member whose id is not a UUID', () => {
    const parsed = memberSchema.safeParse({
      id: 'm1',
      display_name: 'Alice',
      avatar_url: null,
      email: 'alice@example.com',
      goal_mode: 'job_hunting',
      timezone: 'UTC',
      oauth_expires_at: ISO,
      is_restricted: false,
      created_at: ISO,
    });
    expect(parsed.success).toBe(false);
  });

  it('parses an Approval with kb_refs', () => {
    const parsed = approvalSchema.safeParse({
      id: UUID,
      member_id: UUID,
      action_type: 'publish_post',
      tier: 3,
      status: 'pending',
      payload: { preview: 'foo' },
      kb_refs: [
        {
          record_id: UUID,
          title: 'Resume',
          category: 'resume',
          excerpt: 'excerpt',
          url: null,
        },
      ],
      governance: null,
      trace_id: UUID,
      idempotency_key: 'ik-12345678',
      created_at: ISO,
      decided_at: null,
    });
    expect(parsed.success).toBe(true);
  });

  it('rejects an Approval with an out-of-range tier', () => {
    const parsed = approvalSchema.safeParse({
      id: UUID,
      member_id: UUID,
      action_type: 'publish_post',
      tier: 9,
      status: 'pending',
      payload: {},
      kb_refs: [],
      governance: null,
      trace_id: UUID,
      idempotency_key: 'ik-12345678',
      created_at: ISO,
      decided_at: null,
    });
    expect(parsed.success).toBe(false);
  });

  it('parses a ContentItem', () => {
    const parsed = contentItemSchema.safeParse({
      id: UUID,
      member_id: UUID,
      status: 'draft',
      body: 'hello',
      title: null,
      media: [],
      variant: null,
      scheduled_at: null,
      published_at: null,
      trace_id: UUID,
      kb_refs: [],
      created_at: ISO,
      updated_at: ISO,
    });
    expect(parsed.success).toBe(true);
  });

  it('parses a RestrictionState with its canonical reason enum', () => {
    const parsed = restrictionStateSchema.safeParse({
      member_id: UUID,
      is_restricted: true,
      reason: 'governance_fail',
      reason_detail: null,
      triggered_at: ISO,
      cleared_at: null,
    });
    expect(parsed.success).toBe(true);
  });

  it('rejects an unknown restriction reason', () => {
    const parsed = restrictionStateSchema.safeParse({
      member_id: UUID,
      is_restricted: true,
      reason: 'not_a_real_reason',
      reason_detail: null,
      triggered_at: ISO,
      cleared_at: null,
    });
    expect(parsed.success).toBe(false);
  });

  it('parses a queue item', () => {
    const parsed = queueItemSchema.safeParse({
      id: UUID,
      kind: 'followup_target',
      actor: { id: 'c1', display_name: 'Alice', avatar_url: null },
      body: 'Reply to Alice',
      priority: 'normal',
      tags: ['followup'],
      created_at: ISO,
    });
    expect(parsed.success).toBe(true);
  });
});
