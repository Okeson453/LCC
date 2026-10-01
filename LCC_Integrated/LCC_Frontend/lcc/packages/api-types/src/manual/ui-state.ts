/**
 * UI state types — view-model shapes used in the frontend.
 *
 * These live outside `@lcc/api-types/generated/*` because they are
 * frontend-only compositions (e.g., a BriefingCard that combines a
 * `BriefingItem` with a `Contact` and an `Approval`).
 */

import type { ApprovalStatus } from './approval';
import type { RiskTier } from './risk-tier';
import type { KbCitation } from './kb-citation';

export type BriefingSectionKind =
  | 'approvals_due'
  | 'hot_opportunities'
  | 'engagement'
  | 'followups'
  | 'content_suggestions';

export interface BriefingCardViewModel {
  id: string;
  kind: BriefingSectionKind;
  title: string;
  summary: string;
  actionUrl: string;
  tier: RiskTier;
  /** Set when the card is an approval awaiting decision. */
  approvalId: string | null;
  status: ApprovalStatus | null;
  kbRefs: KbCitation[];
}

export interface ComposerVariantViewModel {
  variant: 'authority' | 'contrarian' | 'bts' | 'case_study';
  body: string;
  kbRefs: KbCitation[];
  groundingScore: number;
}

export interface QualityFlagViewModel {
  kind: 'fluff' | 'tone' | 'banned_phrase' | 'length' | 'unsupported_claim' | 'low_grounding';
  severity: 'info' | 'warn' | 'error';
  span: string;
  message: string;
}

export interface OpportunityViewModel {
  id: string;
  kind: 'job' | 'client_lead';
  title: string;
  company: string | null;
  status: string;
  fitScore: number;
  fitBreakdown: Record<string, number>;
  actionPlan: 'apply_now' | 'engage_then_message' | 'warm_intro' | 'watchlist' | 'archive';
  hiddenSignals: string[];
  capturedAt: string;
}

export interface InboxItemViewModel {
  id: string;
  kind: 'dm' | 'comment' | 'connection_request' | 'mention' | 'reaction';
  actor: {
    id: string;
    displayName: string;
    avatarUrl: string | null;
  };
  snippet: string;
  priority: 'low' | 'normal' | 'high' | 'urgent';
  createdAt: string;
  unread: boolean;
}

export interface AccountHealthViewModel {
  score: number;
  components: {
    quotaRemaining: number;
    groundingScore: number;
    approvalThroughput: number;
    recentDenialRate: number;
  };
  quotaConsumedToday: number;
  quotaCapToday: number;
  computedAt: string;
  isRestricted: boolean;
  restrictionReason: string | null;
}
