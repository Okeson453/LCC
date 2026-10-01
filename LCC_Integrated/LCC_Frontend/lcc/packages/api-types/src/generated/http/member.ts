/* eslint-disable */
/**
 * @generated
 * From schemas/openapi/api-gateway.yaml — Member resources.
 */

import type { UUID, DateTime, Email, Uri } from './common';

export type GoalMode = 'job_hunting' | 'client_acquisition' | 'hybrid';
export type Theme = 'light' | 'dark' | 'system' | 'high_contrast';

export interface Member {
  id: UUID;
  email: Email;
  display_name: string;
  avatar_url: Uri | null;
  goal_mode: GoalMode;
  timezone: string;
  oauth_expires_at: DateTime;
  is_restricted: boolean;
  created_at: DateTime;
}

export interface MemberUpdate {
  display_name?: string;
  avatar_url?: Uri;
  timezone?: string;
}

export interface MemberSettings {
  goal_mode?: GoalMode;
  notifications_enabled?: boolean;
  quiet_hours_start?: string | null;
  quiet_hours_end?: string | null;
  theme?: Theme;
}

export interface MemberSettingsUpdate {
  goal_mode?: GoalMode;
  notifications_enabled?: boolean;
  quiet_hours_start?: string;
  quiet_hours_end?: string;
  theme?: Theme;
}

export interface TokenPair {
  access_token: string;
  expires_in: number;
}

export interface JobRef {
  job_id: UUID;
  status: 'queued' | 'running' | 'succeeded' | 'failed';
}
