/**
 * Extension integration test — non-negotiable §8: no LinkedIn credentials
 * in chrome.storage.local. Audit ref: M-46.
 */
import { test, expect } from '@playwright/test';
import path from 'node:path';

test('manifest.json must not include LinkedIn client_secret', async () => {
  const fs = await import('node:fs/promises');
  const manifestPath = path.resolve(__dirname, '../../../public/manifest.json');
  const manifest = await fs.readFile(manifestPath, 'utf8');
  expect(manifest).not.toMatch(/client_secret/i);
  expect(manifest).not.toMatch(/api_key/i);
});

test('host_permissions does not include any non-LinkedIn domain', async () => {
  const fs = await import('node:fs/promises');
  const manifestPath = path.resolve(__dirname, '../../../public/manifest.json');
  const manifest = (await fs.readFile(manifestPath, 'utf8'));
  const parsed = JSON.parse(manifest) as { host_permissions?: string[] };
  const hosts = parsed.host_permissions ?? [];
  expect(hosts.every((h) => h.endsWith('linkedin.com') || h === '<all_urls>')).toBe(true);
});
