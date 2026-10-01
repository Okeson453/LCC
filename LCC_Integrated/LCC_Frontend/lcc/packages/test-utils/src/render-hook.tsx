/**
 * renderHook — wraps `@testing-library/react`'s renderHook with the same
 * providers used by `render`.
 */

import { renderHook as rtlRenderHook, type RenderHookOptions, type RenderHookResult } from '@testing-library/react';
import * as React from 'react';
import { QueryProvider, type QueryProviderProps } from './providers/query-provider';
import { ThemeProvider } from './providers/theme-provider';
import { AuthProvider, type MockSession } from './providers/auth-provider';
import { RealtimeProvider, type RealtimeProviderProps } from './providers/realtime-provider';

export interface CustomRenderHookOptions<TProps> extends Omit<RenderHookOptions<TProps>, 'wrapper'> {
  queryClient?: QueryProviderProps['client'];
  initialSession?: MockSession | null;
  realtimeClient?: RealtimeProviderProps['client'];
}

export function renderHook<TResult, TProps>(
  callback: (props: TProps) => TResult,
  options: CustomRenderHookOptions<TProps> = {},
): RenderHookResult<TResult, TProps> {
  const { queryClient, initialSession = null, realtimeClient, ...rest } = options;

  const Wrapper: React.FC<{ children: React.ReactNode }> = ({ children }) => (
    <QueryProvider client={queryClient}>
      <ThemeProvider>
        <AuthProvider initialSession={initialSession}>
          {realtimeClient ? (
            <RealtimeProvider client={realtimeClient}>{children}</RealtimeProvider>
          ) : (
            children
          )}
        </AuthProvider>
      </ThemeProvider>
    </QueryProvider>
  );

  return rtlRenderHook(callback, { wrapper: Wrapper, ...rest });
}
