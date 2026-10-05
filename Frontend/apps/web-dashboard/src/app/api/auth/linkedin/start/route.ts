import { NextResponse } from 'next/server';
import { randomUUID } from 'node:crypto';
import { buildAuthorizationUrl } from '@/lib/auth/linkedin-oauth';

export const runtime = 'nodejs';

// These handlers are per-request: they read cookies / the request URL and
// server-only env (LINKEDIN_CLIENT_ID, …) at call time. Without
// `force-dynamic` Next tries to prerender them during `next build`, which
// runs the handler at build time and throws
//   [env] "LINKEDIN_CLIENT_ID" is not defined on the server.
// Marking them dynamic is what makes `pnpm build` succeed.
export const dynamic = 'force-dynamic';

export async function GET(): Promise<NextResponse> {
  const state = randomUUID();
  const url = await buildAuthorizationUrl(state);
  const res = NextResponse.redirect(url);
  res.cookies.set('oauth_state', state, {
    httpOnly: true,
    secure: true,
    sameSite: 'lax',
    maxAge: 600,
    path: '/',
  });
  return res;
}
