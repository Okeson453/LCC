import { describe, expect, it } from 'vitest';
import { ApprovalSchema, KbRecordSchema, MemberSchema, OpportunitySchema } from '@lcc/api-types';

describe('zod schemas', () => {
  it('parses a minimal Member', () => {
    const parsed = MemberSchema.safeParse({
      id: 'm1',
      display_name: 'Alice',
      headline: 'PM',
      avatar_url: null,
      email: null,
      role: 'member',
      goal_mode: 'job',
      timezone: 'UTC',
      created_at: new Date().toISOString(),
    });
    expect(parsed.success).toBe(true);
  });

  it('parses an Approval with kb_refs', () => {
    const parsed = ApprovalSchema.safeParse({
      id: 'a1',
      member_id: 'm1',
      action_type: 'publish_post',
      tier: 3,
      payload: { preview: 'foo' },
      target: { type: 'contact', id: 'c1' },
      kb_refs: [],
      trace_id: 'tr1',
      idempotency_key: 'ik1',
      status: 'pending',
      created_at: new Date().toISOString(),
    });
    expect(parsed.success).toBe(true);
  });

  it('rejects invalid Opportunity score', () => {
    const r = OpportunitySchema.safeParse({
      id: 'o1',
      member_id: 'm1',
      title: 'job',
      company: 'acme',
      fit_score: 2,
      evidence: [],
      action_plan: 'apply',
      action_items: [],
      status: 'discovered',
      created_at: new Date().toISOString(),
    });
    expect(r.success).toBe(false);
  });

  it('parses a KbRecord', () => {
    const r = KbRecordSchema.safeParse({
      id: 'k1', member_id: 'm1', kind: 'authority', title: 't', body: 'b', created_at: new Date().toISOString(),
    });
    expect(r.success).toBe(true);
  });
});
