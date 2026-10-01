export * from './components/RestrictedStateBanner';
export * from './components/ComplianceGate';
export * from './components/AccountHealthGauge';
export * from './components/QuotaMeter';
export * from './components/CompliancePosturePanel';

export * from './hooks/use-restricted-state';
export * from './hooks/use-compliance-config';

export * from './context/ComplianceProvider';

export * from './utils/state-derivation';
export * from './logic/derive-state';
export * from './logic/banner-rules';

export const DEFAULT_RESTRICTED_STATE = { restricted: false, reason: null, until: null } as const;
export function classifyRestrictedState(state: { restricted: boolean; reason: string | null }): 'info' | 'warning' | 'danger' {
  if (!state.restricted) return 'info';
  if (state.reason === 'pause' || state.reason === 'gov_block') return 'danger';
  return 'warning';
}
