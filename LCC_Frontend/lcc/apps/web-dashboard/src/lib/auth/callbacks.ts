/**
 * Auth callback helpers — JWT / session manipulation.
 *
 * Used in `next-auth.ts` callbacks and in custom session endpoints.
 */

import type { JWT } from 'next-auth/jwt';
import type { Session } from 'next-auth';

export interface LccSession extends Session {
  accessToken?: string;
  refreshToken?: string;
  user?: {
    id?: string;
    name?: string | null;
    email?: string | null;
    image?: string | null;
  };
}

export interface LccJwt extends JWT {
  accessToken?: string;
  refreshToken?: string;
}

export function readAccessToken(session: Session | null): string | null {
  const s = session as LccSession | null;
  return s?.accessToken ?? null;
}
