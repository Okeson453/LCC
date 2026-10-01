/**
 * Admin API — compliance config versions, restrictions.
 *
 * Canonical paths from `config.ts::API_PATHS.admin`.
 */

import { apiFetch } from './client';
import { API_PATHS } from './config';
import type {
  ComplianceConfigCreate,
  ComplianceConfigVersion,
  RestrictionState,
} from '@lcc/api-types';

export async function listComplianceConfigVersions(): Promise<ComplianceConfigVersion[]> {
  return apiFetch<ComplianceConfigVersion[]>(API_PATHS.admin.listConfigVersions);
}

export async function proposeComplianceConfig(
  input: ComplianceConfigCreate,
): Promise<ComplianceConfigVersion> {
  return apiFetch<ComplianceConfigVersion>(API_PATHS.admin.complianceConfigVersions, {
    method: 'POST',
    body: input,
  });
}

export async function activateComplianceConfig(versionId: string): Promise<ComplianceConfigVersion> {
  return apiFetch<ComplianceConfigVersion>(
    API_PATHS.admin.activateConfig(versionId),
    { method: 'POST' },
  );
}

export async function getRestrictionState(memberId: string): Promise<RestrictionState> {
  return apiFetch<RestrictionState>(`/api/v1/admin/compliance/restrictions/${memberId}`);
}

export async function clearRestriction(memberId: string): Promise<RestrictionState> {
  return apiFetch<RestrictionState>(`/api/v1/admin/compliance/restrictions/${memberId}/clear`, {
    method: 'POST',
  });
}
