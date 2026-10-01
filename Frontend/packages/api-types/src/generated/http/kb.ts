/* eslint-disable */
/**
 * @generated
 * From schemas/openapi/api-gateway.yaml — KB resources.
 */

import type { UUID, DateTime } from './common';

export type KbCategory =
  | 'resume'
  | 'portfolio'
  | 'voice_sample'
  | 'case_study'
  | 'ideal_customer'
  | 'content_pillar'
  | 'goal';

export interface KbRecord {
  id: UUID;
  member_id: UUID;
  category: KbCategory;
  title: string;
  body: string;
  metadata: Record<string, unknown>;
  created_at: DateTime;
  updated_at: DateTime;
}

export interface KbRecordCreate {
  category: KbCategory;
  title: string;
  body: string;
  metadata?: Record<string, unknown>;
}

export interface KbRecordUpdate {
  title?: string;
  body?: string;
  metadata?: Record<string, unknown>;
}
