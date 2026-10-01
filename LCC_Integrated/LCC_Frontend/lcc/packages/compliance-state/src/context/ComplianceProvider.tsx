'use client';

/**
 * ComplianceProvider — context provider that pulls compliance state from
 * the backend (HTTP + WebSocket) and exposes it via context.
 */

import * as React from 'react';
import type { RestrictionStateDTO, ComplianceConfigVersion } from '@lcc/api-types';
import { useComplianceChannel, type RealtimeClient } from '@lcc/realtime';
import { RestrictedStateContextValue, ComplianceContext } from '../hooks/use-restricted-state';
import type { ComplianceConfigContextValue } from '../hooks/use-compliance-config';
import { ComplianceConfigContext } from '../hooks/use-compliance-config';
import { deriveRestrictionState } from '../utils/state-derivation';

export interface ComplianceProviderProps {
  children: React.ReactNode;
  /** RealtimeClient (already configured with token + memberId). */
  realtimeClient: RealtimeClient;
  /** Fetcher for the current restriction state. */
  fetcher: () => Promise<RestrictionStateDTO>;
  /** Fetcher for the active compliance config. */
  configFetcher: () => Promise<ComplianceConfigVersion>;
  /** Auto-refetch interval (ms). Defaults to 60_000. */
  refreshIntervalMs?: number;
}

export function ComplianceProvider({
  children,
  realtimeClient,
  fetcher,
  configFetcher,
  refreshIntervalMs = 60_000,
}: ComplianceProviderProps): React.ReactElement {
  const [raw, setRaw] = React.useState<RestrictionStateDTO | null>(null);
  const [config, setConfig] = React.useState<ComplianceConfigVersion | null>(null);

  const refetchRestriction = React.useCallback(async () => {
    try {
      const next = await fetcher();
      setRaw(next);
    } catch (e) {
      // Surface to telemetry but don't crash the app.
      // eslint-disable-next-line no-console
      console.warn('[compliance-state] refetch failed:', (e as Error).message);
    }
  }, [fetcher]);

  const refetchConfig = React.useCallback(async () => {
    try {
      const next = await configFetcher();
      setConfig(next);
    } catch (e) {
      // eslint-disable-next-line no-console
      console.warn('[compliance-state] config refetch failed:', (e as Error).message);
    }
  }, [configFetcher]);

  // Initial fetch + interval
  React.useEffect(() => {
    void refetchRestriction();
    void refetchConfig();
    const id = setInterval(() => {
      void refetchRestriction();
      void refetchConfig();
    }, refreshIntervalMs);
    return () => clearInterval(id);
  }, [refetchRestriction, refetchConfig, refreshIntervalMs]);

  // Subscribe to WS events for instant updates
  const { data } = useComplianceChannel(realtimeClient);
  React.useEffect(() => {
    if (data && data.payload.memberId === raw?.member_id) {
      setRaw((prev) =>
        prev
          ? {
              ...prev,
              is_restricted: data.payload.isRestricted,
              reason: data.payload.reason,
              reason_detail: data.payload.reasonDetail,
              triggered_at: data.payload.triggeredAt,
            }
          : null,
      );
    }
  }, [data, raw?.member_id]);

  const restrictedValue = React.useMemo<RestrictedStateContextValue>(
    () => ({
      raw,
      derived: deriveRestrictionState(raw),
      refetch: refetchRestriction,
    }),
    [raw, refetchRestriction],
  );

  const configValue = React.useMemo<ComplianceConfigContextValue>(
    () => ({
      config,
      refetch: refetchConfig,
    }),
    [config, refetchConfig],
  );

  return (
    <ComplianceContext.Provider value={restrictedValue}>
      <ComplianceConfigContext.Provider value={configValue}>
        {children}
      </ComplianceConfigContext.Provider>
    </ComplianceContext.Provider>
  );
}
