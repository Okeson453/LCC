/* eslint-disable */
/**
 * @generated
 * From schemas/openapi/api-gateway.yaml — Network (CRM) resources.
 */

import type { UUID, DateTime, Uri } from './common';

export type RelationshipStage =
  | 'cold'
  | 'connected'
  | 'engaged'
  | 'conversation'
  | 'opportunity'
  | 'closed';

export type InteractionKind = 'dm' | 'comment' | 'email' | 'meeting' | 'intro' | 'like' | 'connection';

export interface ContactCompany {
  id: string;
  name: string;
}

export interface Contact {
  id: UUID;
  member_id: UUID;
  display_name: string;
  avatar_url: Uri | null;
  headline: string;
  company: ContactCompany | null;
  stage: RelationshipStage;
  tags: string[];
  last_interaction_at: DateTime | null;
  warmth_score: number;
  created_at: DateTime;
}

export interface ContactUpdate {
  tags?: string[];
  stage?: RelationshipStage;
  notes?: string;
}

export interface Interaction {
  id: UUID;
  kind: InteractionKind;
  body: string;
  occurred_at: DateTime;
  created_at: DateTime;
  direction: 'inbound' | 'outbound';
}

export interface InteractionCreate {
  kind: InteractionKind;
  body?: string;
  occurred_at: DateTime;
  direction?: 'inbound' | 'outbound';
}
