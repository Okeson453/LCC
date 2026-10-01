/**
 * Zod schemas for runtime validation of API responses.
 *
 * These mirror the OpenAPI-generated types but provide runtime validation
 * (defends against backend contract drift). The codegen script
 * `codegen/generate-zod.ts` re-generates these from OpenAPI extensions; this
 * file is the canonical hand-written version that codegen overwrites.
 */

import { z } from 'zod';

// ─── Common ─────────────────────────────────────────────────────────────────
const uuid = z.string().uuid();
const dateTime = z.string().datetime({ offset: true });
const email = z.string().email();

const memberIdSchema = uuid;

// ─── Member ─────────────────────────────────────────────────────────────────
export const memberSchema = z.object({
  id: memberIdSchema,
  email,
  display_name: z.string().min(1),
  avatar_url: z.string().url().nullable(),
  goal_mode: z.enum(['job_hunting', 'client_acquisition', 'hybrid']),
  timezone: z.string(),
  oauth_expires_at: dateTime,
  is_restricted: z.boolean(),
  created_at: dateTime,
});
export type Member = z.infer<typeof memberSchema>;

export const memberUpdateSchema = z.object({
  display_name: z.string().min(1).optional(),
  avatar_url: z.string().url().optional(),
  timezone: z.string().optional(),
});

export const memberSettingsSchema = z.object({
  goal_mode: z.enum(['job_hunting', 'client_acquisition', 'hybrid']).optional(),
  notifications_enabled: z.boolean().optional(),
  quiet_hours_start: z.string().nullable().optional(),
  quiet_hours_end: z.string().nullable().optional(),
  theme: z.enum(['light', 'dark', 'system', 'high_contrast']).optional(),
});

export const tokenPairSchema = z.object({
  access_token: z.string(),
  expires_in: z.number().int().positive(),
});

// ─── Briefing ───────────────────────────────────────────────────────────────
export const briefingSectionSchema = z.object({
  kind: z.enum(['approvals_due', 'hot_opportunities', 'engagement', 'followups', 'content_suggestions']),
  title: z.string(),
  items: z.array(z.object({
    id: uuid,
    kind: z.string(),
    title: z.string(),
    summary: z.string(),
    action_url: z.string(),
    tier: z.number().int().min(1).max(5),
  })),
});
export const briefingSchema = z.object({
  date: z.string(),
  sections: z.array(briefingSectionSchema),
  generated_at: dateTime,
});

// ─── Approval ───────────────────────────────────────────────────────────────
export const approvalSchema = z.object({
  id: uuid,
  member_id: memberIdSchema,
  action_type: z.enum([
    'publish_post',
    'send_connection',
    'send_dm',
    'send_message',
    'apply_opportunity',
    'send_proposal',
    'edit_profile',
    'comment',
    'like',
  ]),
  tier: z.number().int().min(1).max(5),
  status: z.enum(['pending', 'approved', 'rejected', 'expired', 'failed']),
  payload: z.record(z.unknown()),
  kb_refs: z.array(z.object({
    record_id: uuid,
    title: z.string(),
    category: z.string(),
    excerpt: z.string(),
    url: z.string().url().nullable(),
  })),
  governance: z.object({
    permit: z.boolean(),
    failed_guard: z.string(),
    reason: z.string(),
  }).nullable(),
  trace_id: uuid,
  idempotency_key: z.string(),
  created_at: dateTime,
  decided_at: dateTime.nullable(),
});
export type ApprovalDTO = z.infer<typeof approvalSchema>;

export const approvalDecisionSchema = z.object({
  decision: z.enum(['approve', 'reject']),
  edited_payload: z.record(z.unknown()).optional(),
  comment: z.string().optional(),
});

// ─── Profile ────────────────────────────────────────────────────────────────
export const profileSnapshotSchema = z.object({
  id: uuid,
  member_id: memberIdSchema,
  captured_at: dateTime,
  strength: z.number().int().min(0).max(100),
  components: z.object({
    headline: z.number().int().min(0).max(100),
    about: z.number().int().min(0).max(100),
    experience: z.number().int().min(0).max(100),
    skills: z.number().int().min(0).max(100),
    network: z.number().int().min(0).max(100),
  }),
});

// ─── Content ────────────────────────────────────────────────────────────────
export const contentItemSchema = z.object({
  id: uuid,
  member_id: memberIdSchema,
  status: z.enum(['draft', 'pending_approval', 'scheduled', 'publishing', 'published', 'rejected']),
  body: z.string(),
  title: z.string().nullable(),
  media: z.array(z.object({
    id: uuid,
    kind: z.enum(['image', 'video', 'document']),
    url: z.string().url(),
    alt: z.string().nullable(),
  })),
  variant: z.string().nullable(),
  scheduled_at: dateTime.nullable(),
  published_at: dateTime.nullable(),
  trace_id: uuid,
  kb_refs: z.array(z.object({
    record_id: uuid,
    title: z.string(),
    category: z.string(),
    excerpt: z.string(),
    url: z.string().url().nullable(),
  })),
  created_at: dateTime,
  updated_at: dateTime,
});
export type ContentItemDTO = z.infer<typeof contentItemSchema>;

// ─── Engagement ─────────────────────────────────────────────────────────────
export const queueItemSchema = z.object({
  id: uuid,
  kind: z.enum(['comment_target', 'congratulation_target', 'followup_target']),
  actor: z.object({
    id: z.string(),
    display_name: z.string(),
    avatar_url: z.string().url().nullable(),
  }),
  body: z.string(),
  priority: z.enum(['low', 'normal', 'high']),
  tags: z.array(z.string()),
  created_at: dateTime,
});

// ─── Compliance ─────────────────────────────────────────────────────────────
export const restrictionStateSchema = z.object({
  member_id: memberIdSchema,
  is_restricted: z.boolean(),
  reason: z.enum(['denial_rate', 'manual_review', 'oauth_expired', 'governance_fail', 'abuse_signal', 'none']),
  reason_detail: z.string().nullable(),
  triggered_at: dateTime.nullable(),
  cleared_at: dateTime.nullable(),
});
export type RestrictionStateDTO = z.infer<typeof restrictionStateSchema>;

// ─── Analytics ──────────────────────────────────────────────────────────────
export const accountHealthSchema = z.object({
  score: z.number().min(0).max(1),
  components: z.object({
    quota_remaining: z.number(),
    grounding_score: z.number(),
    approval_throughput: z.number(),
    recent_denial_rate: z.number(),
  }),
  quota_consumed_today: z.number().int().nonnegative(),
  quota_cap_today: z.number().int().nonnegative(),
  computed_at: dateTime,
  is_restricted: z.boolean(),
  restriction_reason: z.string().nullable(),
});
export type AccountHealthDTO = z.infer<typeof accountHealthSchema>;

// ─── Error envelopes ────────────────────────────────────────────────────────
export const apiErrorSchema = z.object({
  error: z.object({
    code: z.string(),
    message: z.string(),
    details: z.record(z.unknown()).optional(),
  }),
  trace_id: uuid,
});

export const guardFailureSchema = z.object({
  guard: z.string(),
  reason: z.string(),
  trace_id: uuid,
});
export type GuardFailureDTO = z.infer<typeof guardFailureSchema>;

// ─── Validation helpers ─────────────────────────────────────────────────────
export class ApiContractError extends Error {
  readonly code = 'API_CONTRACT_ERROR';
  constructor(
    readonly zodError: z.ZodError,
    readonly url: string,
    readonly status: number,
  ) {
    super(
      `API contract violation at ${url} (HTTP ${status}): ${zodError.issues
        .map((i) => `${i.path.join('.')}: ${i.message}`)
        .join('; ')}`,
    );
  }
}

/**
 * Validate an API response against its zod schema. Throws ApiContractError on
 * mismatch — callers should catch and surface to telemetry (Sentry) with
 * trace_id so backend team can investigate.
 */
export function validateApiResponse<T>(
  schema: z.ZodType<T>,
  data: unknown,
  context: { url: string; status: number; traceId: string },
): T {
  const result = schema.safeParse(data);
  if (!result.success) {
    throw new ApiContractError(result.error, context.url, context.status);
  }
  return result.data;
}
