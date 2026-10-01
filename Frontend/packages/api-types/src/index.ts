/**
 * @lcc/api-types — public exports.
 *
 * Apps import from this package directly. Never reach into `./generated/*`.
 *
 * NOTE: branches are exported in priority order; branches that would
 * re-export an already-exported name use explicit re-exports instead, so the
 * canonical definitions live in ./generated/http.
 */
export * from './generated/http';
export * from './generated/grpc/compliance/governor';
export * from './generated/grpc/compliance/admin';
export * from './generated/grpc/intelligence/ai';
export * from './generated/grpc/intelligence/opportunity';
export * from './generated/grpc/intelligence/kb';
export * from './generated/grpc/intelligence/voice';
export * from './generated/grpc/intelligence/scoring';
export * from './generated/grpc/events/envelope';
export type { ComplianceRestrictionDetectedEvent } from './generated/events';
export type { ContentItemApprovedEvent } from './generated/events';
export type { MemberCreatedEvent } from './generated/events';
export type { AccountHealthViewModel } from './manual';
export type { ActionItem } from './manual';
export type { ApprovalDecisionPayload } from './manual';
export type { ApprovalDecisionResponse } from './manual';
export type { ApprovalTarget } from './manual';
export type { ApprovalViewModel } from './manual';
export type { BriefingCardViewModel } from './manual';
export type { ComposerVariantViewModel } from './manual';
export type { GovernanceDecision } from './manual';
export type { InboxItemViewModel } from './manual';
export type { KbCitation } from './manual';
export type { OpportunityEvidence } from './manual';
export type { OpportunityStage } from './manual';
export type { OpportunityViewModel } from './manual';
export type { QualityFlagViewModel } from './manual';
export type { RiskTier } from './manual';
export type { RiskTierDescriptor } from './manual';
export type { TracedRequest } from './manual';
export type { TracedResponse } from './manual';
export { ACTION_TYPE_LABELS, ACTION_TYPE_TO_TIER, KB_CATEGORY_LABELS, RISK_TIERS, STAGE_TIER_THRESHOLD, TIER_DESCRIPTORS, getTierDescriptor, groupCitationsByCategory, isValidCitationList, tierAllowsEdit, tierForActionType, tierRequiresApproval } from './manual';
export type { AccountHealthDTO } from './runtime';
export type { ApprovalDTO } from './runtime';
export type { ApprovalId } from './runtime';
export type { CompanyId } from './runtime';
export type { ComplianceConfigVersionId } from './runtime';
export type { ContactId } from './runtime';
export type { ContentItemDTO } from './runtime';
export type { ContentItemId } from './runtime';
export type { GuardFailureDTO } from './runtime';
export type { IdempotencyKey } from './runtime';
export type { InboxItemId } from './runtime';
export type { InteractionId } from './runtime';
export type { KbRecordId } from './runtime';
export type { MemberId } from './runtime';
export type { OpportunityId } from './runtime';
export type { QueueItemId } from './runtime';
export type { RestrictionStateDTO } from './runtime';
export type { SequenceId } from './runtime';
export type { SequenceStepId } from './runtime';
export type { TraceId } from './runtime';
export { ApiContractError, accountHealthSchema, apiErrorSchema, approvalDecisionSchema, approvalId, approvalSchema, briefingSchema, briefingSectionSchema, companyId, complianceConfigVersionId, contactId, contentItemId, contentItemSchema, generateIdempotencyKey, generateTraceId, generateUuidV4, guardFailureSchema, idempotencyKey, inboxItemId, interactionId, isIdempotencyKey, isMemberId, isTraceId, kbRecordId, memberId, memberSchema, memberSettingsSchema, memberUpdateSchema, opportunityId, permitToken, profileSnapshotSchema, queueItemId, queueItemSchema, restrictionStateSchema, sequenceId, sequenceStepId, tokenPairSchema, traceId, validateApiResponse } from './runtime';
