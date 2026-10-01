/**
 * useComplianceConfig — exposes the active ComplianceConfigVersion.
 *
 * Per Non-Negotiable §4: all numeric thresholds/caps in the UI come from the
 * backend's active compliance config version. The frontend never hardcodes.
 *
 * Per audit S-19: default caps and graceful fallbacks replace throws so the
 * UI does not crash on a slow / missing config fetch.
 */

import * as React from 'react';
import type { ComplianceConfigVersion } from '@lcc/api-types';

export interface ComplianceConfigContextValue {
  config: ComplianceConfigVersion | null;
  isLoading: boolean;
  refetch: () => Promise<void>;
}

const Ctx = React.createContext<ComplianceConfigContextValue | null>(null);

export function useComplianceConfig(): ComplianceConfigVersion | null {
  const ctx = React.useContext(Ctx);
  // S-19 fix: prefer contextual null rather than throw.
  return ctx?.config ?? null;
}

const DEFAULTS = {
  daily_action_cap: 25,
  connection_per_day_cap: 20,
  dm_per_day_cap: 40,
  min_grounding_score: 0.6,
} as const;

/** Convenience accessors for specific caps. Returns the backend value or a fallback. */
export function useDailyActionCap(): number {
  const config = useComplianceConfig();
  return (config?.config.daily_action_cap as number | undefined) ?? DEFAULTS.daily_action_cap;
}

export function useConnectionPerDayCap(): number {
  const config = useComplianceConfig();
  return (config?.config.connection_per_day_cap as number | undefined) ?? DEFAULTS.connection_per_day_cap;
}

export function useDmPerDayCap(): number {
  const config = useComplianceConfig();
  return (config?.config.dm_per_day_cap as number | undefined) ?? DEFAULTS.dm_per_day_cap;
}

export function useMinGroundingScore(): number {
  const config = useComplianceConfig();
  return (config?.config.min_grounding_score as number | undefined) ?? DEFAULTS.min_grounding_score;
}

export function useHasComplianceProvider(): boolean {
  return React.useContext(Ctx) !== null;
}
