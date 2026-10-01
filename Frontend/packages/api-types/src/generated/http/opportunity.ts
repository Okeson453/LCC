/* eslint-disable */
/**
 * @generated
 * From schemas/openapi/api-gateway.yaml — Opportunity resources.
 */

import type { UUID, DateTime, Uri } from './common';
import type { KbCitationRef } from './profile';

export type OpportunityKind = 'job' | 'client_lead';
export type OpportunityStatus =
  | 'discovered'
  | 'qualified'
  | 'drafting'
  | 'applied'
  | 'interviewing'
  | 'offer'
  | 'won'
  | 'lost'
  | 'parked';
export type OpportunityActionPlan =
  | 'apply_now'
  | 'engage_then_message'
  | 'warm_intro'
  | 'watchlist'
  | 'archive';

export interface OpportunityFitBreakdown {
  skills: number;
  seniority: number;
  domain: number;
  location: number;
  comp: number;
  culture: number;
}

export interface Opportunity {
  id: UUID;
  member_id: UUID;
  title: string;
  kind: OpportunityKind;
  company: string;
  url: Uri | null;
  status: OpportunityStatus;
  fit_score: number;
  fit_breakdown: OpportunityFitBreakdown;
  action_plan: OpportunityActionPlan;
  captured_at: DateTime;
}

export interface OpportunityDetail extends Opportunity {
  description: string;
  requirements: string[];
  posted_at: DateTime;
  hidden_signals: string[];
}

export interface ApplicationDraft {
  id: UUID;
  body: string;
  attachments: string[];
  kb_refs: KbCitationRef[];
}

export interface ProposalDraft {
  id: UUID;
  body: string;
  fee_estimate: string | null;
  timeline: string | null;
  kb_refs: KbCitationRef[];
}
