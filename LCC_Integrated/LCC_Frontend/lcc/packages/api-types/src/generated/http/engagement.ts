/* eslint-disable */
/**
 * @generated
 * From schemas/openapi/api-gateway.yaml — Engagement resources.
 */

import type { UUID, DateTime, Uri } from './common';

export type InboxKind = 'dm' | 'comment' | 'connection_request' | 'mention' | 'reaction';
export type InboxPriority = 'low' | 'normal' | 'high' | 'urgent';
export type QueueKind = 'comment_target' | 'congratulation_target' | 'followup_target';

export interface InboxActor {
  id: string;
  display_name: string;
  avatar_url: Uri | null;
}

export interface InboxItem {
  id: UUID;
  kind: InboxKind;
  actor: InboxActor;
  snippet: string;
  priority: InboxPriority;
  created_at: DateTime;
  unread: boolean;
}

export interface QueueItem {
  id: UUID;
  kind: QueueKind;
  actor: InboxActor;
  body: string;
  priority: 'low' | 'normal' | 'high';
  tags: string[];
  created_at: DateTime;
}

export interface ReplyVariant {
  tone: 'warm' | 'professional' | 'brief' | 'assertive';
  body: string;
}

export interface ReplyDecision {
  variant_index: number;
  body: string;
}
