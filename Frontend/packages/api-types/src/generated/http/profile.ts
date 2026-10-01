/* eslint-disable */
/**
 * @generated
 * From schemas/openapi/api-gateway.yaml — Profile resources.
 */

import type { UUID, DateTime } from './common';

export interface ProfileComponents {
  headline: number;
  about: number;
  experience: number;
  skills: number;
  network: number;
}

export interface ProfileSnapshot {
  id: UUID;
  member_id: UUID;
  captured_at: DateTime;
  strength: number;
  components: ProfileComponents;
}

export interface EditDraft {
  id: UUID;
  field: string;
  current_value: string;
  suggested_value: string;
  rationale: string;
  priority: number;
  kb_refs: KbCitationRef[];
}

export interface ProfileStrengthPoint {
  captured_at: DateTime;
  strength: number;
  delta?: number;
}

export interface KbCitationRef {
  record_id: UUID;
  title: string;
  category: string;
  excerpt: string;
  url: string | null;
}
