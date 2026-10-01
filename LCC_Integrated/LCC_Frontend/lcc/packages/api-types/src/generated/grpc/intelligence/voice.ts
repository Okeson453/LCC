/* eslint-disable */
/**
 * @generated
 * From proto/intelligence/voice.proto
 */

import type { UUID, DateTime } from '../compliance/_common';

export interface VoiceSample {
  id: UUID;
  memberId: UUID;
  text: string;
  source: 'linkedin_post' | 'manual_paste' | 'interview';
  capturedAt: DateTime;
}

export interface VoiceProfile {
  memberId: UUID;
  tone: string;
  bannedPhrases: string[];
  preferredOpenings: string[];
  preferredClosings: string[];
  signaturePatterns: string[];
  updatedAt: DateTime;
}
