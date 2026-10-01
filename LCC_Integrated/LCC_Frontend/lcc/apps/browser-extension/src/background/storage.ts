/**
 * Token + config storage. Single source of truth used by both auth.ts and
 * the popup. Audit ref: M-21.
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
  return ConfigSchema.parse(stored ?? { apiBase: 'http://localhost:8080', popMode: 'panel', redactBeforeLog: true });
}

export async function setConfig(next: Config): Promise<void> {
  ConfigSchema.parse(next);
  await chrome.storage.local.set({ [KEY_CONFIG]: next });
}

export async function getAccessToken(): Promise<string | null> {
  const stored = (await chrome.storage.local.get([KEY_TOKEN]))[KEY_TOKEN] as unknown;
  if (!stored) return null;
  const parsed = TokenSchema.safeParse(stored);
  if (!parsed.success) return null;
  if (parsed.data.expiresAt <= Date.now()) return null;
  return parsed.data.accessToken;
}

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
