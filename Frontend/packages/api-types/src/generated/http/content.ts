/* eslint-disable */
/**
 * @generated
 * From schemas/openapi/api-gateway.yaml — Content resources.
 */

import type { UUID, DateTime, Uri } from './common';
import type { KbCitationRef } from './profile';

export type ContentStatus =
  | 'draft'
  | 'pending_approval'
  | 'scheduled'
  | 'publishing'
  | 'published'
  | 'rejected';

export type ContentVariantKind = 'authority' | 'contrarian' | 'bts' | 'case_study';

export interface MediaItem {
  id: UUID;
  kind: 'image' | 'video' | 'document';
  url: Uri;
  alt: string | null;
}

export interface ContentItem {
  id: UUID;
  member_id: UUID;
  status: ContentStatus;
  body: string;
  title: string | null;
  media: MediaItem[];
  variant: string | null;
  scheduled_at: DateTime | null;
  published_at: DateTime | null;
  trace_id: UUID;
  kb_refs: KbCitationRef[];
  created_at: DateTime;
  updated_at: DateTime;
}

export interface ContentCreate {
  body: string;
  title?: string;
  variant?: string;
  media_ids?: UUID[];
}

export interface ContentUpdate {
  body?: string;
  title?: string;
  media_ids?: UUID[];
}

export interface ContentVariant {
  variant: ContentVariantKind;
  body: string;
  kb_refs: KbCitationRef[];
}

export interface ComposeRequest {
  prompt: string;
  variants?: ContentVariantKind[];
  pillar_id?: UUID | null;
}

export interface QualityFlag {
  kind: 'fluff' | 'tone' | 'banned_phrase' | 'length' | 'unsupported_claim' | 'low_grounding';
  severity: 'info' | 'warn' | 'error';
  span: string;
  message: string;
}

export interface QualityReport {
  score: number;
  flags: QualityFlag[];
  kb_coverage: number;
}

export interface CalendarEntry {
  date: string;
  items: ContentItem[];
}

export interface ScheduleRequest {
  scheduled_at: DateTime;
}
