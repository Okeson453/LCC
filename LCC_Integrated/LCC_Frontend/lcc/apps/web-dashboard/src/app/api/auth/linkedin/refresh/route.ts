import { NextRequest, NextResponse } from 'next/server';
import { refreshLinkedInToken } from '@/lib/auth/linkedin-oauth';

export const runtime = 'nodejs';

export async function POST(req: NextRequest): Promise<NextResponse> {
  return refreshLinkedInToken(req);
}
