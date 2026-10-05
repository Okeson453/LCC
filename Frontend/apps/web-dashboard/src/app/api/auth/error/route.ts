import { NextRequest, NextResponse } from 'next/server';

// These handlers are per-request: they read cookies / the request URL and
// server-only env (LINKEDIN_CLIENT_ID, …) at call time. Without
// `force-dynamic` Next tries to prerender them during `next build`, which
// runs the handler at build time and throws
//   [env] "LINKEDIN_CLIENT_ID" is not defined on the server.
// Marking them dynamic is what makes `pnpm build` succeed.
export const dynamic = 'force-dynamic';


export async function GET(req: NextRequest): Promise<NextResponse> {
  const url = new URL('/(auth)/error', req.nextUrl.origin);
  url.searchParams.set('reason', req.nextUrl.searchParams.get('reason') ?? 'unknown');
  return NextResponse.redirect(url);
}
