/**
 * KB API — records (CRUD).
 *
 * Canonical paths from `config.ts::API_PATHS.kb`.
 */

import { apiFetch } from './client';
import { API_PATHS } from './config';
import type { KbCategory, KbRecord, KbRecordCreate, KbRecordUpdate } from '@lcc/api-types';

export async function listKbRecords(_memberId: string, category?: KbCategory): Promise<KbRecord[]> {
  return apiFetch<KbRecord[]>(API_PATHS.kb.records, { query: { kind: category } });
}

export async function createKbRecord(_memberId: string, input: KbRecordCreate): Promise<KbRecord> {
  return apiFetch<KbRecord>(API_PATHS.kb.records, { method: 'POST', body: input });
}

export async function updateKbRecord(
  _memberId: string,
  recordId: string,
  input: KbRecordUpdate,
): Promise<KbRecord> {
  return apiFetch<KbRecord>(API_PATHS.kb.record(recordId), {
    method: 'PATCH',
    body: input,
  });
}

export async function deleteKbRecord(_memberId: string, recordId: string): Promise<void> {
  await apiFetch<void>(API_PATHS.kb.record(recordId), { method: 'DELETE' });
}

export async function getKbRecord(_memberId: string, recordId: string): Promise<KbRecord> {
  return apiFetch<KbRecord>(API_PATHS.kb.record(recordId));
}

export async function reembedKbRecord(_memberId: string, recordId: string): Promise<void> {
  await apiFetch<void>(API_PATHS.kb.reembed(recordId), { method: 'POST' });
}
