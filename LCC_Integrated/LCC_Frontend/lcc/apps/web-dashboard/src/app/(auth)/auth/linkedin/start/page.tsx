/**
 * /auth/linkedin/start — Begin OAuth handshake.
 *
 * Server-side: redirect to backend's /auth/linkedin/start with state + PKCE.
 */

import { redirect } from 'next/navigation';
import { newTraceId } from '@/lib/auth/trace-id';

export default function LinkedInStart({
  searchParams,
}: {
  searchParams: { return_to?: string };
}): never {
  const apiBase = process.env.NEXT_PUBLIC_API_BASE ?? '';
  const returnTo = searchParams.return_to ?? '/today';
  const state = newTraceId();
  const url = new URL(`${apiBase}/auth/linkedin/start`);
  url.searchParams.set('state', state);
  url.searchParams.set('return_to', returnTo);
  redirect(url.toString());
}
