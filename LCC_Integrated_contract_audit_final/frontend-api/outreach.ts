/**
 * Outreach API — sequences, steps, templates.
 *
 * Canonical paths from `config.ts::API_PATHS.sequences`.
 */

import { apiFetch } from './client';
import { API_PATHS } from './config';
import type {
  Sequence,
  SequenceCreate,
  SequenceDetail,
  SequenceStep,
  SequenceTemplate,
  StepApprovalRequest,
} from '@lcc/api-types';

export async function listSequences(_memberId: string): Promise<Sequence[]> {
  return apiFetch<Sequence[]>(API_PATHS.sequences.list);
}

export async function createSequence(_memberId: string, input: SequenceCreate): Promise<Sequence> {
  return apiFetch<Sequence>(API_PATHS.sequences.list, { method: 'POST', body: input });
}

export async function getSequence(_memberId: string, sequenceId: string): Promise<SequenceDetail> {
  return apiFetch<SequenceDetail>(`/api/v1/sequences/${sequenceId}`);
}

export async function approveStep(
  _memberId: string,
  _sequenceId: string,
  stepId: string,
  _request: StepApprovalRequest,
): Promise<SequenceStep> {
  return apiFetch<SequenceStep>(API_PATHS.sequences.stepSent(stepId), {
    method: 'POST',
  });
}

export async function pauseSequence(_memberId: string, sequenceId: string): Promise<Sequence> {
  return apiFetch<Sequence>(API_PATHS.sequences.pause(sequenceId), {
    method: 'PATCH',
    body: { version: 0, reason: 'user_requested' },
  });
}

export async function listSequenceTemplates(_memberId: string): Promise<SequenceTemplate[]> {
  return apiFetch<SequenceTemplate[]>(API_PATHS.sequences.templates);
}
