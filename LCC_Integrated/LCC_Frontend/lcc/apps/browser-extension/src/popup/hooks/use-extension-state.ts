/**
 * Hooks for popup state — list of pending approvals, badge number.
 * Audit ref: M-32.
 */
import * as React from 'react';
import { useQuery } from '@tanstack/react-query';
import { getAccessToken } from '../lib/storage';

export function useExtensionToken(): { hasToken: boolean | null } {
  const q = useQuery({ queryKey: ['ext', 'token'], queryFn: () => getAccessToken() });
  return { hasToken: q.data === null ? false : q.data !== undefined ? true : null };
}

export function useRestrictedState(memberId: string | null) {
  return useQuery({
    queryKey: ['restricted', memberId],
    enabled: Boolean(memberId),
    queryFn: async () => null as { restricted: boolean; reason: string | null; until: string | null } | null,
  });
}

export function useSyncEffect(fn: () => void, deps: React.DependencyList): void {
  // eslint-disable-next-line react-hooks/exhaustive-deps
  React.useEffect(fn, deps);
}
