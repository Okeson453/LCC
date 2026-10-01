/**
 * /auth/refresh — silent token refresh.
 *
 * Server-side route: forward to backend /auth/refresh, set new cookie, return.
 */

import { NextResponse } from 'next/server';

export async function GET(): Promise<Response> {
  const apiBase = process.env.NEXT_PUBLIC_API_BASE ?? '';
  const res = await fetch(`${apiBase}/auth/refresh`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    cache: 'no-store',
  });
  if (!res.ok) return new NextResponse('refresh failed', { status: res.status });
  return new NextResponse(null, { status: 204 });
}
