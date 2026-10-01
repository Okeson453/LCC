import { NextRequest, NextResponse } from 'next/server';
import { handleAuthorizationCallback } from '@/lib/auth/linkedin-oauth';

export const runtime = 'nodejs';

export async function GET(req: NextRequest): Promise<NextResponse> {
  return handleAuthorizationCallback(req);
}
