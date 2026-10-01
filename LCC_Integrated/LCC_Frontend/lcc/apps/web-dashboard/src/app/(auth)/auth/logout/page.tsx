/**
 * /auth/logout — revoke refresh + LinkedIn grant, clear cookie, redirect.
 */

import { redirect } from 'next/navigation';

export default async function LogoutPage(): Promise<never> {
  const apiBase = process.env.NEXT_PUBLIC_API_BASE ?? '';
  await fetch(`${apiBase}/auth/logout`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    cache: 'no-store',
  }).catch(() => null);
  redirect('/auth/linkedin/start');
}
