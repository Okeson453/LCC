import { NextRequest, NextResponse } from 'next/server';

export async function GET(req: NextRequest): Promise<NextResponse> {
  const url = new URL('/(auth)/error', req.nextUrl.origin);
  url.searchParams.set('reason', req.nextUrl.searchParams.get('reason') ?? 'unknown');
  return NextResponse.redirect(url);
}
