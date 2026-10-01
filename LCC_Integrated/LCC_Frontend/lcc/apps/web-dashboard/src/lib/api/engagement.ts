/**
 * Engagement API — inbox, queue, reply, approve.
 *
 * Canonical paths from `config.ts::API_PATHS.engagement`.
 */

import { apiFetch } from './client';
import { API_PATHS } from './config';
import type { InboxItem, QueueItem, ReplyDecision, ReplyVariant } from '@lcc/api-types';

export async function fetchInbox(_memberId: string, cursor?: string): Promise<InboxItem[]> {
  return apiFetch<InboxItem[]>(API_PATHS.engagement.inbox, { query: { cursor } });
}

export async function fetchQueue(_memberId: string): Promise<QueueItem[]> {
  return apiFetch<QueueItem[]>(API_PATHS.engagement.queue);
}

export async function draftReply(_memberId: string, taskId: string): Promise<ReplyVariant[]> {
  return apiFetch<ReplyVariant[]>(API_PATHS.engagement.draft(taskId), {
    method: 'PATCH',
    body: { version: 0, draft: '', draft_pins: [] },
  });
}

export async function approveReply(
  _memberId: string,
  taskId: string,
  decision: ReplyDecision,
): Promise<QueueItem> {
  return apiFetch<QueueItem>(API_PATHS.engagement.complete(taskId), {
    method: 'POST',
    body: { version: 0, ...decision },
  });
}
