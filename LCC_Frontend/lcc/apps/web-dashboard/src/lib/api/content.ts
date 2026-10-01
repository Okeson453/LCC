/**
 * Content API — items, compose, quality check, schedule, calendar.
 *
 * Canonical paths from `config.ts::API_PATHS.content`.
 * NOTE: canonical namespace is `/api/v1/content/items` — content is no
 * longer nested under `/members/{member_id}/content`.
 */

import { apiFetch } from './client';
import { API_PATHS } from './config';
import type {
  CalendarEntry,
  ComposeRequest,
  ContentCreate,
  ContentItem,
  ContentUpdate,
  ContentVariant,
  QualityReport,
} from '@lcc/api-types';

export async function listContent(
  _memberId: string,
  status?: string,
): Promise<ContentItem[]> {
  return apiFetch<ContentItem[]>(API_PATHS.content.items, { query: { status } });
}

export async function createContent(_memberId: string, input: ContentCreate): Promise<ContentItem> {
  return apiFetch<ContentItem>(API_PATHS.content.items, { method: 'POST', body: input });
}

export async function getContentItem(_memberId: string, contentId: string): Promise<ContentItem> {
  return apiFetch<ContentItem>(API_PATHS.content.item(contentId));
}

export async function updateContentItem(
  _memberId: string,
  contentId: string,
  input: ContentUpdate,
): Promise<ContentItem> {
  return apiFetch<ContentItem>(API_PATHS.content.item(contentId), {
    method: 'PATCH',
    body: input,
  });
}

export async function deleteContentItem(_memberId: string, contentId: string): Promise<void> {
  await apiFetch<void>(API_PATHS.content.item(contentId), { method: 'DELETE' });
}

export async function composeContent(
  _memberId: string,
  input: ComposeRequest,
): Promise<ContentVariant[]> {
  return apiFetch<ContentVariant[]>('/api/v1/content/items/compose', {
    method: 'POST',
    body: input,
  });
}

export async function runQualityCheck(_memberId: string, contentId: string): Promise<QualityReport> {
  return apiFetch<QualityReport>(API_PATHS.content.qualityCheck(contentId), {
    method: 'POST',
  });
}

export async function submitForApproval(_memberId: string, contentId: string): Promise<ContentItem> {
  // Submit-for-approval transitions to `in_review` and creates an approval record.
  return apiFetch<ContentItem>(API_PATHS.content.transition(contentId), {
    method: 'POST',
    body: { version: 0, new_state: 'in_review' },
  });
}

export async function scheduleContent(
  _memberId: string,
  contentId: string,
  scheduledAt: string,
): Promise<ContentItem> {
  return apiFetch<ContentItem>(API_PATHS.content.schedule(contentId), {
    method: 'POST',
    body: { scheduled_at: scheduledAt, slots: [] },
  });
}

export async function fetchContentCalendar(
  _memberId: string,
  from?: string,
  to?: string,
): Promise<CalendarEntry[]> {
  return apiFetch<CalendarEntry[]>('/api/v1/content/calendar', {
    query: { from, to },
  });
}
