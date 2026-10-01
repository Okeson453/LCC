/**
 * Opaque brand types.
 *
 * Use brand types to distinguish IDs that are nominally strings at the wire
 * level but semantically distinct at the type level. The compiler will refuse
 * to pass a `ContactId` where a `MemberId` is expected, even though both
 * are strings at runtime.
 */

declare const brand: unique symbol;
type Brand<T, B> = T & { readonly [brand]: B };

export type MemberId = Brand<string, 'MemberId'>;
export type ContactId = Brand<string, 'ContactId'>;
export type CompanyId = Brand<string, 'CompanyId'>;
export type ContentItemId = Brand<string, 'ContentItemId'>;
export type ApprovalId = Brand<string, 'ApprovalId'>;
export type SequenceId = Brand<string, 'SequenceId'>;
export type SequenceStepId = Brand<string, 'SequenceStepId'>;
export type OpportunityId = Brand<string, 'OpportunityId'>;
export type KbRecordId = Brand<string, 'KbRecordId'>;
export type ComplianceConfigVersionId = Brand<string, 'ComplianceConfigVersionId'>;
export type InteractionId = Brand<string, 'InteractionId'>;
export type QueueItemId = Brand<string, 'QueueItemId'>;
export type InboxItemId = Brand<string, 'InboxItemId'>;
export type TraceId = Brand<string, 'TraceId'>;
export type IdempotencyKey = Brand<string, 'IdempotencyKey'>;
export type PermitToken = Brand<string, 'PermitToken'>;

const UUID_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
const UUID_V4_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
const IDEMPOTENCY_KEY_RE = /^[A-Za-z0-9_-]{8,128}$/;

function isUuid(s: string): boolean {
  return UUID_RE.test(s);
}

/** Throws if the value is not a UUID-shaped string. */
function brandAs<T extends string>(value: string, type: string): T {
  if (!isUuid(value)) {
    throw new TypeError(`Invalid ${type}: expected UUID, got "${value}"`);
  }
  return value as T;
}

export const memberId = (s: string): MemberId => brandAs<MemberId>(s, 'MemberId');
export const contactId = (s: string): ContactId => brandAs<ContactId>(s, 'ContactId');
export const companyId = (s: string): CompanyId => brandAs<CompanyId>(s, 'CompanyId');
export const contentItemId = (s: string): ContentItemId => brandAs<ContentItemId>(s, 'ContentItemId');
export const approvalId = (s: string): ApprovalId => brandAs<ApprovalId>(s, 'ApprovalId');
export const sequenceId = (s: string): SequenceId => brandAs<SequenceId>(s, 'SequenceId');
export const sequenceStepId = (s: string): SequenceStepId => brandAs<SequenceStepId>(s, 'SequenceStepId');
export const opportunityId = (s: string): OpportunityId => brandAs<OpportunityId>(s, 'OpportunityId');
export const kbRecordId = (s: string): KbRecordId => brandAs<KbRecordId>(s, 'KbRecordId');
export const complianceConfigVersionId = (s: string): ComplianceConfigVersionId =>
  brandAs<ComplianceConfigVersionId>(s, 'ComplianceConfigVersionId');
export const interactionId = (s: string): InteractionId => brandAs<InteractionId>(s, 'InteractionId');
export const queueItemId = (s: string): QueueItemId => brandAs<QueueItemId>(s, 'QueueItemId');
export const inboxItemId = (s: string): InboxItemId => brandAs<InboxItemId>(s, 'InboxItemId');

/** Trace IDs are UUID v4 (more strict than MemberId). */
export function traceId(s: string): TraceId {
  if (!UUID_V4_RE.test(s)) {
    throw new TypeError(`Invalid TraceId: expected UUID v4, got "${s}"`);
  }
  return s as TraceId;
}

/** Idempotency keys are base64url-ish strings (8–128 chars). */
export function idempotencyKey(s: string): IdempotencyKey {
  if (!IDEMPOTENCY_KEY_RE.test(s)) {
    throw new TypeError(`Invalid IdempotencyKey: got "${s}"`);
  }
  return s as IdempotencyKey;
}

/** Permit tokens are opaque strings issued by the Compliance Governor. */
export function permitToken(s: string): PermitToken {
  if (s.length < 16) {
    throw new TypeError('PermitToken too short');
  }
  return s as PermitToken;
}

/** Type guards (no throw). */
export function isMemberId(s: string): s is MemberId {
  return isUuid(s);
}
export function isTraceId(s: string): s is TraceId {
  return UUID_V4_RE.test(s);
}
export function isIdempotencyKey(s: string): s is IdempotencyKey {
  return IDEMPOTENCY_KEY_RE.test(s);
}

/** Generate a fresh UUID v4 (browser crypto). */
export function generateUuidV4(): string {
  if (typeof crypto !== 'undefined' && crypto.randomUUID) {
    return crypto.randomUUID();
  }
  // Fallback (e.g., Node.js test environments)
  // eslint-disable-next-line @typescript-eslint/no-var-requires
  const nodeCrypto = require('crypto') as typeof import('crypto');
  return nodeCrypto.randomUUID();
}

export function generateIdempotencyKey(): IdempotencyKey {
  const u = generateUuidV4();
  return idempotencyKey(u);
}

export function generateTraceId(): TraceId {
  return traceId(generateUuidV4());
}
