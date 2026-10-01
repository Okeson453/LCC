/* eslint-disable */
/**
 * @generated
 * From proto/intelligence/kb.proto
 */

import type { UUID, DateTime } from '../compliance/_common';

export interface KbIngestRequest {
  memberId: UUID;
  source: 'upload' | 'linkedin_consent' | 'paste' | 'voice_interview';
  content: string;
  category: string;
  metadata: Record<string, unknown>;
}

export interface KbIngestResponse {
  recordId: UUID;
  traceId: UUID;
  ingestedAt: DateTime;
  warnings: string[];
}
