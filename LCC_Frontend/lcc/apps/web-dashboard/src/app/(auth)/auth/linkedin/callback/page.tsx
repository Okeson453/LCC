/**
 * /auth/linkedin/callback — OAuth callback handler.
 *
 * Server-side: forwards code + state to backend, sets HTTP-only cookie,
 * then redirects to the originally-requested route (or /today).
 *
 * Fixes per audit S-04: replaced `throw new Error(...)` with redirects to
 * a user-friendly error page.
 */

import { redirect } from 'next/navigation';
import { headers } from 'next/headers';
import { API_BASE } from '@/lib/api/config';

export default async function LinkedInCallback({
  searchParams,
}: {
  searchParams: { code?: string; state?: string; return_to?: string };
}): Promise<never> {
  if (!searchParams.code || !searchParams.state) {
    redirect('/auth/error?reason=missing_params');
  }

  const traceId = headers().get('x-trace-id') ?? '';
  const apiBase = API_BASE;

  let res: Response;
  try {
    res = await fetch(
      `${apiBase}/auth/linkedin/callback?code=${encodeURIComponent(searchParams.code!)}&state=${encodeURIComponent(searchParams.state!)}`,
      {
        method: 'GET',
        headers: { 'x-trace-id': traceId, accept: 'application/json' },
        redirect: 'manual',
        cache: 'no-store',
      },
    );
  } catch (err) {
    redirect(`/auth/error?reason=network&detail=${encodeURIComponent(String(err))}`);
  }

  if (res.status === 0 || res.status >= 500) {
    redirect('/auth/error?reason=server_error');
  }
  if (res.status === 401 || res.status === 403) {
    redirect('/auth/error?reason=unauthorized');
  }
  if (res.status === 422 || res.status === 409) {
    redirect(`/auth/error?reason=invalid_state`);
  }

  const dest = searchParams.return_to ?? '/onboarding';
  redirect(dest);
}
