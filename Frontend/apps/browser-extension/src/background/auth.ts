/**
 * Extension auth — exchanges a deep-linked OAuth code for an access token via
 * the dashboard's `/api/v1/extensions/exchange` endpoint. Per audit M-18.
 *
 * Tokens are stored in chrome.storage.local only (NEVER chrome.storage.sync)
 * and only set when the response carries the backend-issued access token.
 */
import { setAccessToken, setRefreshToken, getConfig, getRefreshToken } from '../lib/storage';

export interface TokenBundle {
  access_token: string;
  refresh_token?: string;
  expires_in: number;
  member_id: string;
}

export async function exchangeExtensionToken(code: string): Promise<TokenBundle> {
  const cfg = await getConfig();
  const url = `${cfg.apiBase}/api/v1/extensions/exchange`;
  const res = await fetch(url, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', 'x-trace-id': crypto.randomUUID() },
    body: JSON.stringify({ code, client_id: cfg.apiBase }),
  });
  if (!res.ok) throw new Error('exchange_failed');
  const json = (await res.json()) as TokenBundle;
  if (!json.access_token) throw new Error('exchange_missing_token');
  await setAccessToken(json.access_token, json.expires_in);
  if (json.refresh_token) await setRefreshToken(json.refresh_token);
  return json;
}

export async function refreshExtensionToken(): Promise<TokenBundle> {
  const refresh = await getRefreshToken();
  if (!refresh) throw new Error('no_refresh_token');
  const cfg = await getConfig();
  const url = `${cfg.apiBase}/api/v1/extensions/refresh`;
  const res = await fetch(url, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', 'x-trace-id': crypto.randomUUID() },
    body: JSON.stringify({ refresh_token: refresh }),
  });
  if (!res.ok) throw new Error('refresh_failed');
  const json = (await res.json()) as TokenBundle;
  if (!json.access_token) throw new Error('refresh_missing_token');
  await setAccessToken(json.access_token, json.expires_in);
  if (json.refresh_token) await setRefreshToken(json.refresh_token);
  return json;
}

export async function invalidateToken(): Promise<void> {
  await setAccessToken(null);
}
