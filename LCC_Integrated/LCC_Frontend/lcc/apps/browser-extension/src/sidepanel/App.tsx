/**
 * Sidepanel React app. Audit ref: M-33.
 */
import * as React from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { SidePanelShell } from './components/SidePanelShell';

const qc = new QueryClient({
  defaultOptions: { queries: { staleTime: 30_000, retry: 1, refetchOnWindowFocus: false } },
});

export function App(): React.ReactElement {
  return (
    <QueryClientProvider client={qc}>
      <SidePanelShell />
    </QueryClientProvider>
  );
}
