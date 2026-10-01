/**
 * Profile API — snapshot, audit, edit drafts, strength history.
 *
 * Canonical paths from `config.ts::API_PATHS.profile`.
 */

import { apiFetch } from './client';
import { API_PATHS } from './config';
import type {
  EditDraft,
  JobRef,
  ProfileSnapshot,
  ProfileStrengthPoint,
} from '@lcc/api-types';

export async function fetchLatestSnapshot(_memberId: string): Promise<ProfileSnapshot> {
  return apiFetch<ProfileSnapshot>(API_PATHS.profile.me);
}

export async function runProfileAudit(_memberId: string): Promise<JobRef> {
  return apiFetch<JobRef>('/api/v1/profile/me/audit', { method: 'POST' });
}

export async function generateEditDrafts(_memberId: string): Promise<EditDraft[]> {
  return apiFetch<EditDraft[]>(API_PATHS.profile.editDrafts, { method: 'POST' });
}

export async function fetchStrengthHistory(
  _memberId: string,
  range: '7d' | '30d' | '90d' | '365d' = '90d',
): Promise<ProfileStrengthPoint[]> {
  return apiFetch<ProfileStrengthPoint[]>('/api/v1/profile/me/strength-history', {
    query: { range },
  });
}

export interface ProfileExperienceItem { title?: string | null; company?: string | null; start_date?: string | null; end_date?: string | null; current?: boolean; description?: string | null; }
export interface ProfileView { id: string; member_id: string; display_name: string; headline: string; about: string; experience: ProfileExperienceItem[]; skills: string[]; strength: number; components: Record<string, number>; captured_at: string; }

export async function getProfile(memberId: string): Promise<ProfileView> {
  const snap = await apiFetch<ProfileSnapshot>(API_PATHS.profile.me);
  return {
    id: snap.id,
    member_id: snap.member_id,
    display_name: snap.full_name ?? '',
    headline: snap.headline,
    about: snap.summary,
    experience: [],
    skills: snap.skills,
    strength: 0,
    components: {},
    captured_at: snap.created_at,
  };
}
