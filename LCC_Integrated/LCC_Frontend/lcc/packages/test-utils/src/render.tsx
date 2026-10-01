/**
 * Custom render function — wraps with Query + Theme + Auth providers.
 */

import * as React from 'react';
import { render as rtlRender, type RenderOptions, type RenderResult } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryProvider, type QueryProviderProps } from './providers/query-provider';
import { ThemeProvider } from './providers/theme-provider';
import { AuthProvider, type MockSession } from './providers/auth-provider';
import { RealtimeProvider, type RealtimeProviderProps } from './providers/realtime-provider';
import { ApprovalDialogProvider } from '@lcc/approval-gate';

export interface CustomRenderOptions extends Omit<RenderOptions, 'wrapper'> {
  queryClient?: QueryProviderProps['client'];
  initialSession?: MockSession | null;
  realtimeClient?: RealtimeProviderProps['client'];
  /** When true, wraps in `<ApprovalDialogProvider>` (default true). */
  withApprovalDialog?: boolean;
}

export function render(
  ui: React.ReactElement,
  options: CustomRenderOptions = {},
): RenderResult & { user: ReturnType<typeof userEvent.setup> } {
  const {
    queryClient,
    initialSession = null,
    realtimeClient,
    withApprovalDialog = true,
    ...rtlOptions
  } = options;

  const Wrapper: React.FC<{ children: React.ReactNode }> = ({ children }) => (
    <QueryProvider client={queryClient}>
      <ThemeProvider>
        <AuthProvider initialSession={initialSession}>
          {realtimeClient ? (
            <RealtimeProvider client={realtimeClient}>{children}</RealtimeProvider>
          ) : (
            children
          )}
          {withApprovalDialog ? <ApprovalDialogProvider>{null}</ApprovalDialogProvider> : null}
        </AuthProvider>
      </ThemeProvider>
    </QueryProvider>
  );

  const user = userEvent.setup();
  const result = rtlRender(ui, { wrapper: Wrapper, ...rtlOptions });
  return { ...result, user };
}
