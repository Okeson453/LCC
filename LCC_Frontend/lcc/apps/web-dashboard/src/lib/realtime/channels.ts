/**
 * Channel → query-key mapping.
 *
 * 5 channels per `contract_audit/realtime/lcc-realtime-contract.yaml`.
 * The prior `integration` channel was scaffold-only and removed in the
 * canonical cut.
 */

export const CHANNEL_QUERY_KEYS = {
  briefing: ['briefing'] as const,
  approvals: ['approvals'] as const,
  engagement: ['engagement'] as const,
  compliance: ['compliance'] as const,
  sequence: ['outreach', 'sequences'] as const,
};
