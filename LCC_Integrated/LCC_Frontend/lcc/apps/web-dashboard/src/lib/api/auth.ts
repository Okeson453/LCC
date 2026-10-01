/**
 * Auth API — token refresh, logout, OAuth start URL.
 *
 * Canonical paths from `config.ts::API_PATHS.auth` — never inline.
 */

import { apiFetch } from './client';
import { API_PATHS } from './config';
import type { TokenPair } from '@lcc/api-types';

export async function refreshSession(): Promise<TokenPair> {
  return apiFetch<TokenPair>(API_PATHS.auth.refresh, { method: 'POST' });
}

export async function logout(): Promise<void> {
  await apiFetch<void>(API_PATHS.auth.logout, { method: 'POST' });
}

export function linkedInStartUrl(returnTo?: string): string {
  const params = new URLSearchParams();
  if (returnTo) params.set('return_to', returnTo);
  const qs = params.toString();
  return `${API_PATHS.auth.linkedinStart}${qs ? `?${qs}` : ''}`;
}
