/* eslint-disable */
/**
 * @generated
 * From proto/intelligence/scoring.proto
 */

import type { UUID } from '../compliance/_common';

export interface FitScoreRequest {
  memberId: UUID;
  opportunityId: UUID;
  jobDescription: string;
  requirements: string[];
}

export interface FitScoreBreakdown {
  skills: number;
  seniority: number;
  domain: number;
  location: number;
  comp: number;
  culture: number;
}

export interface FitScoreResponse {
  overall: number;
  breakdown: FitScoreBreakdown;
  rationale: string;
  hiddenSignals: string[];
}
