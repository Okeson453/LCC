/* eslint-disable */
/**
 * @generated
 * From schemas/openapi/api-gateway.yaml — Briefing resources.
 */

import type { UUID, DateTime } from './common';

export type BriefingSectionKind =
  | 'approvals_due'
  | 'hot_opportunities'
  | 'engagement'
  | 'followups'
  | 'content_suggestions';

export interface BriefingItem {
  id: UUID;
  kind: string;
  title: string;
  summary: string;
  action_url: string;
  tier: 1 | 2 | 3 | 4 | 5;
}

export interface BriefingSection {
  kind: BriefingSectionKind;
  title: string;
  items: BriefingItem[];
}

export interface Briefing {
  date: string;
  sections: BriefingSection[];
  generated_at: DateTime;
}
