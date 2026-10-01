/**
 * Next.js middleware — auth, i18n, headers.
 *
 * - Adds `x-trace-id` header to every request (forwarded to backend).
 * - Refreshes session JWT if it's near expiry.
 * - Routes /auth/* and /api/auth/* skip auth checks.
 * - Locale negotiation for non-API routes.
 */

import { NextResponse, type NextRequest } from 'next/server';
import { v4 as uuidv4 } from 'uuid';

const PUBLIC_PATHS = [
  '/auth/linkedin/start',
  '/auth/linkedin/callback',
  '/auth/refresh',
  '/auth/error',
  '/auth/logout',
  '/api/auth/linkedin/callback',
  '/manifest.json',
  '/manifest.webmanifest',
  '/icon',
  '/opengraph-image',
  '/twitter-image',
  '/robots.txt',
  '/sitemap.xml',
];

const RESTRICTED_PATHS = ['/restricted'];

function isPublicPath(pathname: string): boolean {
  return PUBLIC_PATHS.some((p) => pathname === p || pathname.startsWith(`${p}/`));
}

function traceIdHeader(req: NextRequest): string {
  return req.headers.get('x-trace-id') ?? uuidv4();
}

export function middleware(req: NextRequest): NextResponse {
  const { pathname } = req.nextUrl;
  const traceId = traceIdHeader(req);

  // Inject trace ID for every request
  const headers = new Headers(req.headers);
  headers.set('x-trace-id', traceId);

  // Public paths bypass auth
  if (isPublicPath(pathname)) {
    return NextResponse.next({ request: { headers } });
  }

  // Auth check via cookie (HTTP-only session cookie set by backend)
  const sessionToken = req.cookies.get('session')?.value;
  if (!sessionToken) {
    const url = req.nextUrl.clone();
    url.pathname = '/auth/linkedin/start';
    url.searchParams.set('return_to', pathname);
    return NextResponse.redirect(url);
  }

  // Forward to backend with trace ID
  const response = NextResponse.next({ request: { headers } });
  response.headers.set('x-trace-id', traceId);
  return response;
}

export const config = {
  matcher: ['/((?!_next/static|_next/image|favicon.ico|public|.*\\..*).*)'],
};
