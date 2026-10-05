/**
 * Token + config storage — single source of truth for the whole extension.
 *
 * Audit ref M-21. Every consumer (fetcher, api-client, ws-client, auth,
 * restricted-state-watcher, popup state) imports from here, so this module
 * owns the storage keys and the token envelope shape.
 *
 * This previously existed twice — here and at `background/storage.ts` — with
 * the same export names but different storage keys (`accessToken` vs
 * `auth.access`), different token envelopes (bare string vs
 * `{ accessToken, expiresAt }`) and different `setAccessToken` arity. Only
 * this file's variant was imported, so `auth.ts` calling
 * `setAccessToken(token, expires_in)` failed to compile against the one-arg
 * signature, and a token written by one module would have been invisible to
 * the other. The richer, expiry-aware implementation is kept here and the
 * duplicate has been removed.
 */
import { z } from 'zod';

const TokenSchema = z.object({
  accessToken: z.string().min(8),
  expiresAt: z.number(),
});

const ConfigSchema = z.object({
  apiBase: z.string().url(),
  popMode: z.enum(['panel', 'popup']),
  redactBeforeLog: z.boolean(),
});

export type Config = z.infer<typeof ConfigSchema>;

const KEY_TOKEN = 'auth.access';
const KEY_REFRESH = 'auth.refresh';
const KEY_CONFIG = 'cfg';

export async function getConfig(): Promise<Config> {
  const stored = (await chrome.storage.local.get([KEY_CONFIG]))[KEY_CONFIG];
  return ConfigSchema.parse(
    stored ?? { apiBase: 'http://localhost:8080', popMode: 'panel', redactBeforeLog: true },
  );
}

export async function setConfig(next: Config): Promise<void> {
  ConfigSchema.parse(next);
  await chrome.storage.local.set({ [KEY_CONFIG]: next });
}

/** Returns the stored access token, or null when absent, malformed or expired. */
export async function getAccessToken(): Promise<string | null> {
  const stored = (await chrome.storage.local.get([KEY_TOKEN]))[KEY_TOKEN] as unknown;
  if (!stored) return null;
  const parsed = TokenSchema.safeParse(stored);
  if (!parsed.success) return null;
  if (parsed.data.expiresAt <= Date.now()) return null;
  return parsed.data.accessToken;
}

/** Stores the access token together with its absolute expiry. Pass null to clear. */
export async function setAccessToken(token: string | null, expiresInSec?: number): Promise<void> {
  if (token === null) {
    await chrome.storage.local.remove(KEY_TOKEN);
    return;
  }
  await chrome.storage.local.set({
    [KEY_TOKEN]: { accessToken: token, expiresAt: Date.now() + (expiresInSec ?? 3600) * 1000 },
  });
}

export async function getRefreshToken(): Promise<string | null> {
  return ((await chrome.storage.local.get([KEY_REFRESH]))[KEY_REFRESH] as string | undefined) ?? null;
}

export async function setRefreshToken(token: string): Promise<void> {
  await chrome.storage.local.set({ [KEY_REFRESH]: token });
}
