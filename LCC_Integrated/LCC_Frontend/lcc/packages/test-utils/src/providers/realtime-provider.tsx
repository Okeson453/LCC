'use client';

/**
 * RealtimeProvider — mock WS client.
 */

import * as React from 'react';
import type { RealtimeClient } from '@lcc/realtime';

export interface RealtimeProviderProps {
  children: React.ReactNode;
  /** Provide a stub RealtimeClient. */
  client: RealtimeClient;
}

const RealtimeContext = React.createContext<RealtimeClient | null>(null);

export function RealtimeProvider({
  children,
  client,
}: RealtimeProviderProps): React.ReactElement {
  return <RealtimeContext.Provider value={client}>{children}</RealtimeContext.Provider>;
}

export function useMockRealtime(): RealtimeClient {
  const ctx = React.useContext(RealtimeContext);
  if (!ctx) throw new Error('useMockRealtime must be used within RealtimeProvider');
  return ctx;
}

export const RealtimeContextKey = RealtimeContext;
