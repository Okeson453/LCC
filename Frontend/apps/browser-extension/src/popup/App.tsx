/**
 * Popup React app — entry used by main.tsx. Audit ref: M-31.
 */
import * as React from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { PopupQueue } from './components/PopupQueue';

const qc = new QueryClient({
  defaultOptions: { queries: { staleTime: 30_000, retry: 1, refetchOnWindowFocus: false } },
});

export function App(): React.ReactElement {
  return (
    <QueryClientProvider client={qc}>
      <PopupQueue />
    </QueryClientProvider>
  );
}
