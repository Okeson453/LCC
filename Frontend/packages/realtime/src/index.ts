export * from './client';
export * from './connection';
export * from './sse-fallback';
export * from './polling-fallback';
export * from './envelope';

export * from './channels/briefing';
export * from './channels/approvals';
export * from './channels/engagement';
export * from './channels/compliance';
export * from './channels/sequence';
export * from './channels/integration';

export * from './hooks/use-channel';
export * from './hooks/use-briefing';
export * from './hooks/use-approvals';
export * from './hooks/use-engagement';
export * from './hooks/use-compliance';
export * from './hooks/use-sequence';

export { useBriefing as useBriefingChannel } from './hooks/use-briefing';
export { useCompliance as useComplianceChannel } from './hooks/use-compliance';
export { useApprovals as useApprovalQueue } from './hooks/use-approvals';
export * from './utils/backoff';
export * from './utils/heartbeat';
export * from './utils/reconnection-state';
