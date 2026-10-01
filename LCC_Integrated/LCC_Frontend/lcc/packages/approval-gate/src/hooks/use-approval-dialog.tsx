'use client';

/**
 * useApprovalDialog — imperative API to open the ApprovalDialog from
 * anywhere in the app. Uses a React context to expose an `open(props)` method.
 *
 * ```tsx
 * const { open } = useApprovalDialog();
 * <Button onClick={() => open({ approvalId: '...', ... })}>Approve</Button>
 * ```
 */

import * as React from 'react';

export interface OpenApprovalDialogInput {
  approvalId: string;
  actionType:
    | 'publish_post'
    | 'send_connection'
    | 'send_dm'
    | 'send_message'
    | 'apply_opportunity'
    | 'send_proposal'
    | 'edit_profile'
    | 'comment'
    | 'like';
  tier: 1 | 2 | 3 | 4 | 5;
  preview: string;
  targetLabel: string;
  kbRefs: readonly import('@lcc/api-types').KbCitation[];
  traceId: string;
  idempotencyKey: string;
  onApprove: (input: {
    editedPreview?: string;
    comment?: string;
  }) => void | Promise<void>;
  onReject: (input: { comment?: string }) => void | Promise<void>;
}

export interface ApprovalDialogApi {
  open: (input: OpenApprovalDialogInput) => void;
  close: () => void;
  isOpen: boolean;
  current: OpenApprovalDialogInput | null;
}

const ApprovalDialogContext = React.createContext<ApprovalDialogApi | null>(null);

export interface ApprovalDialogProviderProps {
  children: React.ReactNode;
}

export function ApprovalDialogProvider({
  children,
}: ApprovalDialogProviderProps): React.ReactElement {
  const [current, setCurrent] = React.useState<OpenApprovalDialogInput | null>(null);

  const open = React.useCallback((input: OpenApprovalDialogInput) => {
    setCurrent(input);
  }, []);

  const close = React.useCallback(() => {
    setCurrent(null);
  }, []);

  const api = React.useMemo<ApprovalDialogApi>(
    () => ({ open, close, isOpen: current !== null, current }),
    [open, close, current],
  );

  return (
    <ApprovalDialogContext.Provider value={api}>
      {children}
    </ApprovalDialogContext.Provider>
  );
}

export function useApprovalDialog(): ApprovalDialogApi {
  const ctx = React.useContext(ApprovalDialogContext);
  if (!ctx) {
    throw new Error('useApprovalDialog must be used within ApprovalDialogProvider');
  }
  return ctx;
}
