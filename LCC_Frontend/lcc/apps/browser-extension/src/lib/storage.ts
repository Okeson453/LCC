import { z } from 'zod';

const ConfigSchema = z.object({
  apiBase: z.string().url(),
  popMode: z.enum(['panel', 'popup']),
  redactBeforeLog: z.boolean(),
});

export type Config = z.infer<typeof ConfigSchema>;

export async function getConfig(): Promise<Config> {
  return ConfigSchema.parse((await chrome.storage.local.get(['cfg'])).cfg ?? { apiBase: 'http://localhost:8080', popMode: 'panel', redactBeforeLog: true });
}

export async function setConfig(next: Config): Promise<void> {
  ConfigSchema.parse(next);
  await chrome.storage.local.set({ cfg: next });
}

export async function getAccessToken(): Promise<string | null> {
  const v = (await chrome.storage.local.get(['accessToken'])).accessToken;
  return typeof v === 'string' ? v : null;
}

export async function setAccessToken(token: string | null): Promise<void> {
  if (token === null) {
    await chrome.storage.local.remove('accessToken');
    return;
  }
  await chrome.storage.local.set({ accessToken: token });
}

export async function getRefreshToken(): Promise<string | null> {
  const v = (await chrome.storage.local.get(['refreshToken'])).refreshToken;
  return typeof v === 'string' ? v : null;
}

export async function setRefreshToken(token: string | null): Promise<void> {
  await chrome.storage.local.set({ refreshToken: token });
}
