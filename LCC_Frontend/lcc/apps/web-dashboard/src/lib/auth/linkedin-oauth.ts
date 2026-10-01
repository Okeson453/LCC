import { randomUUID } from 'node:crypto';
import { cookies } from 'next/headers';
import { NextRequest, NextResponse } from 'next/server';
import { env } from '@/lib/utils/env';

interface AuthorizationState {
  state: string;
  createdAt: number;
  redirectTo?: string;
}

const STATE_TTL_MS = 600_000;

const pendingStates: Map<string, AuthorizationState> = new Map();

export async function buildAuthorizationUrl(state: string, redirectTo?: string): Promise<URL> {
  pendingStates.set(state, { state, createdAt: Date.now(), redirectTo });
  const url = new URL('https://www.linkedin.com/oauth/v2/authorization');
  url.searchParams.set('response_type', 'code');
  url.searchParams.set('client_id', env('LINKEDIN_CLIENT_ID'));
  url.searchParams.set('redirect_uri', env('LINKEDIN_REDIRECT_URI'));
  url.searchParams.set('scope', env('LINKEDIN_SCOPES'));
  url.searchParams.set('state', state);
  return url;
}

export async function handleAuthorizationCallback(req: NextRequest): Promise<NextResponse> {
  const code = req.nextUrl.searchParams.get('code');
  const state = req.nextUrl.searchParams.get('state');
  const error = req.nextUrl.searchParams.get('error');

  const cookieJar = cookies();
  const cookieState = cookieJar.get('oauth_state')?.value;

  if (error || !code || !state || state !== cookieState) {
    const url = new URL('/api/auth/error', req.nextUrl.origin);
    url.searchParams.set('reason', error ?? 'invalid_state');
    return NextResponse.redirect(url);
  }

  const pending = pendingStates.get(state);
  pendingStates.delete(state);
  if (!pending || Date.now() - pending.createdAt > STATE_TTL_MS) {
    const url = new URL('/api/auth/error', req.nextUrl.origin);
    url.searchParams.set('reason', 'expired_state');
    return NextResponse.redirect(url);
  }

  const tokenRes = await fetch('https://www.linkedin.com/oauth/v2/accessToken', {
    method: 'POST',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    body: new URLSearchParams({
      grant_type: 'authorization_code',
      code,
      client_id: env('LINKEDIN_CLIENT_ID'),
      client_secret: env('LINKEDIN_CLIENT_SECRET'),
      redirect_uri: env('LINKEDIN_REDIRECT_URI'),
    }),
  });

  if (!tokenRes.ok) {
    const url = new URL('/api/auth/error', req.nextUrl.origin);
    url.searchParams.set('reason', 'token_exchange_failed');
    return NextResponse.redirect(url);
  }

  const token = (await tokenRes.json()) as { access_token: string; refresh_token?: string; expires_in: number };
  const sessionId = randomUUID();
  const session = {
    sessionId,
    accessToken: token.access_token,
    refreshToken: token.refresh_token ?? null,
    expiresAt: Date.now() + token.expires_in * 1000,
  };

  const res = NextResponse.redirect(new URL(pending.redirectTo ?? '/today', req.nextUrl.origin));
  res.cookies.set('session', Buffer.from(JSON.stringify(session)).toString('base64'), {
    httpOnly: true,
    secure: true,
    sameSite: 'lax',
    path: '/',
    maxAge: 60 * 60 * 24 * 7,
  });
  res.cookies.delete('oauth_state');
  return res;
}

export async function refreshLinkedInToken(req: NextRequest): Promise<NextResponse> {
  const cookieJar = cookies();
  const raw = cookieJar.get('session')?.value;
  if (!raw) return NextResponse.json({ error: 'no_session' }, { status: 401 });

  const session = JSON.parse(Buffer.from(raw, 'base64').toString('utf-8')) as {
    refreshToken: string | null;
    accessToken: string;
    expiresAt: number;
  };
  if (!session.refreshToken) return NextResponse.json({ error: 'no_refresh_token' }, { status: 400 });

  const tokenRes = await fetch('https://www.linkedin.com/oauth/v2/accessToken', {
    method: 'POST',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    body: new URLSearchParams({
      grant_type: 'refresh_token',
      refresh_token: session.refreshToken,
      client_id: env('LINKEDIN_CLIENT_ID'),
      client_secret: env('LINKEDIN_CLIENT_SECRET'),
    }),
  });
  if (!tokenRes.ok) return NextResponse.json({ error: 'refresh_failed' }, { status: 502 });
  const fresh = (await tokenRes.json()) as { access_token: string; expires_in: number; refresh_token?: string };

  const next = {
    sessionId: crypto.randomUUID(),
    accessToken: fresh.access_token,
    refreshToken: fresh.refresh_token ?? session.refreshToken,
    expiresAt: Date.now() + fresh.expires_in * 1000,
  };
  const res = NextResponse.json({ ok: true });
  res.cookies.set('session', Buffer.from(JSON.stringify(next)).toString('base64'), {
    httpOnly: true,
    secure: true,
    sameSite: 'lax',
    path: '/',
    maxAge: 60 * 60 * 24 * 7,
  });
  return res;
}
