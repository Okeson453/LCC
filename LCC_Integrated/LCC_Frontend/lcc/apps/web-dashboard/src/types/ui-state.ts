/**
 * UI state types — exported for components that consume zustand stores.
 */

export type GoalMode = 'job_hunting' | 'client_acquisition' | 'hybrid';

export type NotificationKind =
  | 'approval_created'
  | 'approval_decided'
  | 'compliance_restriction_changed'
  | 'engagement_received'
  | 'opportunity_matched'
  | 'sequence_reply_detected';
