/* eslint-disable */
/**
 * @generated
 * From schemas/openapi/api-gateway.yaml — Outreach resources.
 */

import type { UUID, DateTime } from './common';
import type { KbCitationRef } from './profile';
import type { Contact } from './network';

export type SequenceStatus = 'draft' | 'active' | 'paused' | 'completed' | 'archived';
export type SequenceStepStatus = 'draft' | 'pending_approval' | 'approved' | 'sent' | 'paused';

export interface Sequence {
  id: UUID;
  member_id: UUID;
  name: string;
  status: SequenceStatus;
  step_count: number;
  enrolled_count: number;
  reply_count: number;
  created_at: DateTime;
  updated_at: DateTime;
}

export interface SequenceStep {
  id: UUID;
  day_offset: number;
  body: string;
  status: SequenceStepStatus;
  scheduled_at: DateTime | null;
  trace_id: UUID;
  kb_refs: KbCitationRef[];
}

export interface SequenceDetail extends Sequence {
  steps: SequenceStep[];
  enrolled_contacts: Contact[];
}

export interface SequenceCreate {
  name: string;
  contact_ids: UUID[];
  template_id: UUID;
  start_at?: DateTime;
}

export interface SequenceTemplate {
  id: UUID;
  name: string;
  persona: string;
  step_count: number;
  body_preview: string;
}

export interface StepApprovalRequest {
  decision: 'approve' | 'reject';
  edited_body?: string;
}
