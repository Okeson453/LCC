import { NextRequest, NextResponse } from 'next/server';
import { randomUUID } from 'node:crypto';
import { buildAuthorizationUrl } from '@/lib/auth/linkedin-oauth';

export const runtime = 'nodejs';

export async function GET(req: NextRequest): Promise<NextResponse> {
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
