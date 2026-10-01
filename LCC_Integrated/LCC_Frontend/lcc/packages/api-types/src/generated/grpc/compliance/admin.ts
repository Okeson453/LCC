/* eslint-disable */
/**
 * @generated
 * From proto/compliance/admin.proto
 *
 * Admin operations on the Compliance Governor: configuration versions,
 * two-reviewer activation, restriction management.
 */

import type { UUID, DateTime } from './_common';

export interface ComplianceConfigVersionDTO {
  id: UUID;
  version: number;
  status: 'draft' | 'proposed' | 'active' | 'retired';
  config: Record<string, unknown>;
  rationale: string;
  reviewers: Array<{
    reviewerId: string;
    decision: 'pending' | 'approved' | 'rejected';
    decidedAt: DateTime | null;
  }>;
  createdAt: DateTime;
  activatedAt: DateTime | null;
}

export interface ActivateConfigRequest {
  versionId: UUID;
  reviewerId: string;
  reviewNotes: string;
}
