/**
 * Public types for the browser extension's background, content, popup, and
 * sidepanel surfaces. Populates the previously-empty `src/types/` directory
 * (per audit finding B-01) and serves as the single import surface for the
 * rest of the extension.
 */

export type {
  ActionRequest,
  ActionResponse,
  ApprovalSummary,
} from '../lib/types';

// Re-export shared types from @lcc/api-types for convenience.
export type {
  Approval,
  ComplianceConfig,
  ComplianceReason,
  KbCitation,
  MemberRestriction,
  Member,
  ProfileSnapshot,
  RelationshipStage,
  RiskTier,
} from '@lcc/api-types';

/**
 * Messages exchanged across chrome.runtime boundaries. Each entry is keyed by
 * the originating context.
 */
export type BackgroundMessage =
  | { from: 'popup' | 'sidepanel' | 'content'; type: 'lcc.openSidePanel' }
  | { from: 'popup' | 'sidepanel'; type: 'lcc.refresh' }
  | { from: 'popup' | 'sidepanel'; type: 'lcc.fetcher'; payload: import('../lib/types').ActionRequest }
  | { from: 'background'; type: 'lcc.badgeUpdate'; count: number }
  | { from: 'background'; type: 'lcc.compliance'; restricted: boolean; reason: string | null };

export type ContentMessage =
  | { from: 'content'; type: 'lcc.detectedSurface'; surface: 'profile' | 'company' | 'feed' | 'inbox' }
  | { from: 'content'; type: 'lcc.actionClicked'; trackId: string; tier: 1 | 2 | 3 | 4 | 5 }
  | { from: 'background'; type: 'lcc.applyRestrictions'; reason: 'cooldown' | 'pause' | 'gov_throttle' };

export interface BridgeEnvelope<T> {
  trace_id: string;
  idempotency_key?: string;
  body: T;
}
