/**
 * Silent JWT refresh — called on 401 from the API client.
 */

import { refreshSession } from '../api/auth';
import { readAccessToken } from './callbacks';
import { getSession } from 'next-auth/react';

let refreshing: Promise<string | null> | null = null;

export async function silentRefresh(): Promise<string | null> {
  if (refreshing) return refreshing;
  refreshing = (async () => {
    try {
      const tokens = await refreshSession();
      // In production, the backend sets a new cookie. The frontend just
      // returns the new access token for the API client to use.
      return tokens.access_token;
    } catch {
      return null;
    } finally {
      refreshing = null;
    }
  })();
  return refreshing;
}

/**
 * Reads the current access token from the NextAuth session.
 * Use this in the API client's `getToken` callback.
 */
export async function currentAccessToken(): Promise<string | null> {
  const session = await getSession();
  return readAccessToken(session);
}
