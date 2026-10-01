import { describe, expect, it, beforeEach } from 'vitest';
import { getConfig, setConfig } from '@/lib/storage';

describe('storage', () => {
  beforeEach(async () => {
    await chrome.storage.local.clear();
  });

  it('round-trips config', async () => {
    await setConfig({ apiBase: 'http://localhost:8080', popMode: 'panel', redactBeforeLog: true });
    const c = await getConfig();
    expect(c.apiBase).toBe('http://localhost:8080');
  });

  it('rejects bad apiBase', async () => {
    await expect(setConfig({ apiBase: 'not-a-url', popMode: 'panel', redactBeforeLog: true })).rejects.toThrow();
  });
});
