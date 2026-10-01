/**
 * Members API — current member, settings, account lifecycle.
 *
 * Canonical paths from `config.ts::API_PATHS.members`.
 */

import { apiFetch } from './client';
import { API_PATHS } from './config';
import type { Member, MemberSettings, MemberSettingsUpdate, MemberUpdate } from '@lcc/api-types';

export async function fetchCurrentMember(): Promise<Member> {
  return apiFetch<Member>(API_PATHS.members.me);
}

export async function updateCurrentMember(input: MemberUpdate): Promise<Member> {
  return apiFetch<Member>(API_PATHS.members.me, { method: 'PATCH', body: input });
}

export async function fetchMemberSettings(): Promise<MemberSettings> {
  return apiFetch<MemberSettings>(API_PATHS.members.settings);
}

export async function updateMemberSettings(input: MemberSettingsUpdate): Promise<MemberSettings> {
  return apiFetch<MemberSettings>(API_PATHS.members.settings, { method: 'PATCH', body: input });
}

export async function deleteMember(): Promise<void> {
  await apiFetch<void>(API_PATHS.members.me, { method: 'DELETE' });
}

export async function requestDataExport(memberId: string): Promise<{ job_id: string; status: string }> {
  return apiFetch(API_PATHS.members.export(memberId), { method: 'GET' });
}
