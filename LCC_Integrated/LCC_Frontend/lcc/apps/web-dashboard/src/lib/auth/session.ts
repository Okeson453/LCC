/**
 * Server-side session helper — wraps NextAuth's getServerSession.
 */

import { getServerSession } from 'next-auth';
import { authOptions } from './next-auth';
import type { LccSession } from './callbacks';

export async function getServerAuthSession(): Promise<LccSession | null> {
  return (await getServerSession(authOptions)) as LccSession | null;
}
