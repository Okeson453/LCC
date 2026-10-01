'use client';

/**
 * QueryProvider — TanStack Query test wrapper.
 */

import * as React from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';

export interface QueryProviderProps {
  children: React.ReactNode;
  /** Override the QueryClient for this test. */
  client?: QueryClient;
}

export function makeTestQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        gcTime: 0,
        staleTime: 0,
        refetchOnWindowFocus: false,
        refetchOnReconnect: false,
      },
      mutations: {
        retry: false,
      },
    },
  });
}

export function QueryProvider({ children, client }: QueryProviderProps): React.ReactElement {
  const memo = React.useMemo(() => client ?? makeTestQueryClient(), [client]);
  return <QueryClientProvider client={memo}>{children}</QueryClientProvider>;
}
