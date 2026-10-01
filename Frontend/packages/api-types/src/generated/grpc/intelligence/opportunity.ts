/* eslint-disable */
/**
 * @generated
 * From proto/intelligence/opportunity.proto
 */

import type { UUID, DateTime } from '../compliance/_common';

export interface DiscoveryRequest {
  memberId: UUID;
  sources: Array<'linkedin_jobs' | 'yc' | 'angellist' | 'rss' | 'manual'>;
  filters: Record<string, unknown>;
  limit: number;
}

export interface DiscoveryResponse {
  jobId: UUID;
  traceId: UUID;
  acceptedAt: DateTime;
}
