import type { ApprovalActionType } from '@lcc/api-types';

export interface ActionRequest {
  type: 'lcc.fetcher';
  method: 'GET' | 'POST';
  path: string;
  body?: unknown;
  traceId: string;
  idempotencyKey?: string;
}

export interface ActionResponse {
  ok: boolean;
  data?: unknown;
  error?: string;
}

export interface ApprovalSummary {
  id: string;
  /**
   * Typed per the canonical `ApprovalActionType` enum. This was a bare
   * `string`, which made every `<ApprovalDialog action_type={…}>` fail to
   * typecheck against `ApprovalDialogProps.actionType`.
   */
  action_type: ApprovalActionType;
  tier: 1 | 2 | 3 | 4 | 5;
  target_label: string;
  body_preview: string;
  trace_id: string;
  idempotency_key: string;
  kb_ref_count: number;
  created_at: string;
}
