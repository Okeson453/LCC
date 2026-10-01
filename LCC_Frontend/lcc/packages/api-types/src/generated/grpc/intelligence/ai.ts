/* eslint-disable */
/**
 * @generated
 * From proto/intelligence/ai.proto
 */

import type { UUID, DateTime } from '../compliance/_common';

export interface AiDraftRequest {
  memberId: UUID;
  prompt: string;
  variantHints: string[];
  pillarId: UUID | null;
  kbRefLimit: number;
}

export interface AiDraftVariant {
  variant: 'authority' | 'contrarian' | 'bts' | 'case_study';
  body: string;
  groundingScore: number;
  kbRefs: UUID[];
}

export interface AiDraftResponse {
  variants: AiDraftVariant[];
  traceId: UUID;
  modelId: string;
  generatedAt: DateTime;
}
