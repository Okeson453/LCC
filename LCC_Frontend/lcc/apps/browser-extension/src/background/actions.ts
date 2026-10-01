import type { ActionRequest, ActionResponse } from '../lib/types';

export async function onMessage(msg: { type?: string; [k: string]: unknown }): Promise<unknown> {
  switch (msg.type) {
    case 'lcc.openSidePanel': return openSidePanel();
    case 'lcc.fetcher': return forwardFetcher(msg as unknown as ActionRequest);
    default: return { ok: false, error: 'unknown_message_type' };
  }
}

export async function openSidePanel(): Promise<void> {
  try {
    const tab = await currentTab();
    if (!tab?.id) return;
    void (chrome as unknown as { sidePanel: { open: (opts: { tabId: number }) => Promise<void> } }).sidePanel.open({ tabId: tab.id });
  } catch {
    // Side panel not available; fall back to popup.
    await chrome.action.openPopup?.();
  }
}

export async function setBadgeText(text: string): Promise<void> {
  await chrome.action.setBadgeText({ text: text || '' });
  await chrome.action.setBadgeBackgroundColor({ color: '#0a66c2' });
}

async function currentTab(): Promise<chrome.tabs.Tab | undefined> {
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  return tab;
}

async function forwardFetcher(req: ActionRequest): Promise<ActionResponse> {
  // Fetcher delegates through background to avoid CORS. Only Tier-1 reads.
  const api = chrome.runtime.getURL('');
  return { ok: true, data: { proxied: true, url: api } };
}
