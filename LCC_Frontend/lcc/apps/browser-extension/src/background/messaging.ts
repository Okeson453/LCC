/**
 * Cross-context message relay. Routes popup/sidepanel/content messages to the
 * right handler. Audit ref: M-28.
 */
import type { BackgroundMessage } from '../types';

export async function relayMessage(msg: unknown, _sender: chrome.runtime.MessageSender): Promise<unknown> {
  if (!msg || typeof msg !== 'object') return { ok: false, error: 'invalid_message' };
  const typed = msg as Partial<BackgroundMessage> & { type?: string };

  switch (typed.type) {
    case 'lcc.openSidePanel': {
      const tab = await currentTab();
      if (tab?.id) {
        try { await (chrome as unknown as { sidePanel: { open: (o: { tabId: number }) => Promise<void> } }).sidePanel.open({ tabId: tab.id }); }
        catch { await chrome.action.openPopup?.(); }
      }
      return { ok: true };
    }
    case 'lcc.refresh':
      return { ok: true };
    default:
      return { ok: false, error: 'unknown_message_type' };
  }
}

async function currentTab(): Promise<chrome.tabs.Tab | undefined> {
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  return tab;
}
