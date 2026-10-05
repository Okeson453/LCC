import '@testing-library/jest-dom/vitest';
import { beforeEach, vi } from 'vitest';

/**
 * Minimal in-memory `chrome.storage.local` mock.
 *
 * The extension's modules call `chrome.storage.local.{get,set,remove,clear}`
 * directly. Nothing defined `chrome` in the test environment, so every test
 * touching storage failed with "chrome is not defined" — the suite had never
 * actually run (vitest was not resolvable from this package).
 */
const store = new Map<string, unknown>();

const area = {
  async get(keys?: string | string[] | null) {
    if (keys === undefined || keys === null) {
      return Object.fromEntries(store);
    }
    const list = Array.isArray(keys) ? keys : [keys];
    const out: Record<string, unknown> = {};
    for (const k of list) {
      if (store.has(k)) out[k] = store.get(k);
    }
    return out;
  },
  async set(items: Record<string, unknown>) {
    for (const [k, v] of Object.entries(items)) store.set(k, v);
  },
  async remove(keys: string | string[]) {
    for (const k of Array.isArray(keys) ? keys : [keys]) store.delete(k);
  },
  async clear() {
    store.clear();
  },
};

beforeEach(() => {
  store.clear();
  Object.defineProperty(globalThis, 'chrome', {
    value: {
      storage: { local: area, session: area, sync: area },
      runtime: {
        getURL: (p: string) => `chrome-extension://test${p}`,
        onMessage: { addListener: vi.fn() },
        onInstalled: { addListener: vi.fn() },
        onStartup: { addListener: vi.fn() },
        lastError: undefined,
      },
      alarms: { onAlarm: { addListener: vi.fn() } },
      action: { onClicked: { addListener: vi.fn() }, setBadgeText: vi.fn(), setBadgeBackgroundColor: vi.fn() },
      tabs: {
        query: vi.fn().mockResolvedValue([]),
        onUpdated: { addListener: vi.fn() },
        onActivated: { addListener: vi.fn() },
        sendMessage: vi.fn().mockResolvedValue(undefined),
      },
    },
    writable: true,
    configurable: true,
  });
});
