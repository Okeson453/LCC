/**
 * Client-side session wrapper — typed useSession hook.
 */

'use client';

import { useSession as useNextAuthSession } from 'next-auth/react';
import { useCallback } from 'react';
import type { LccSession } from './callbacks';

export function useClientSession(): {
  session: LccSession | null;
  status: 'loading' | 'authenticated' | 'unauthenticated';
  accessToken: string | null;
} {
  const { data, status } = useNextAuthSession();
  const session = data as LccSession | null;
  return {
    session,
    status,
    accessToken: session?.accessToken ?? null,
  };
}

export function useAccessToken(): string | null {
  const { accessToken } = useClientSession();
  return accessToken;
}

export function useRequireAccessToken(): () => string {
  // F-Audit-32: the access token was read by calling `useClientSession()`
  // *inside* the useCallback body. Hooks may only be called unconditionally at
  // the top level of a component or custom hook; calling one from inside a
  // callback breaks the Rules of Hooks — React logs "Invalid hook call", and
  // because `useCallback` has an empty dependency array the returned function
  // would otherwise close over the very first render's token and keep
  // returning it after a refresh. The hook is now called at the top level and
  // the token is captured in the dependency array.
  const { accessToken } = useClientSession();

  return useCallback(() => {
    if (!accessToken) {
      // S-10 fix: trigger re-auth rather than throw an uncaught error.
      if (typeof window !== 'undefined') {
        window.location.assign('/api/auth/linkedin/start');
      }
      throw new Error('Not authenticated: redirecting to sign-in');
    }
    return accessToken;
  }, [accessToken]);
}
