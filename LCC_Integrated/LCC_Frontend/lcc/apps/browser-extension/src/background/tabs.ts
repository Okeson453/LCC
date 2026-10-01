/**
 * Tab-management helpers — track LinkedIn tabs, drive side-panel open/close
 * by tab, handle URL change detection.
 * Audit ref: M-22.
 */

const TAB_TAG = 'lcc.linkedin.tab';

export async function onTabActivated(tab: chrome.tabs.Tab): Promise<void> {
  if (!tab.id) return;
  if (!tab.url?.includes('linkedin.com')) return;
  await chrome.tabs.update(tab.id, { highlighted: true });
  await openSidePanelForTab(tab.id);
}

export async function trackLinkedInTabs(): Promise<void> {
  const tabs = await chrome.tabs.query({ url: ['https://www.linkedin.com/*'] });
  for (const t of tabs) {
    if (t.id) await tagLinkedInTab(t.id);
  }
  chrome.tabs.onUpdated.addListener(async (id, change, t) => {
    if (t.url?.includes('linkedin.com')) await tagLinkedInTab(id);
    else await untagLinkedInTab(id);
  });
  chrome.tabs.onRemoved.addListener((id) => void untagLinkedInTab(id));
}

async function tagLinkedInTab(id: number): Promise<void> {
  try { await chrome.scripting?.insertCSS?.({ target: { tabId: id }, files: ['content/content.css'] }); } catch { /* ignore */ }
}

async function untagLinkedInTab(_id: number): Promise<void> {
  // No-op. We don't remove styles — they're idempotent.
}

async function openSidePanelForTab(tabId: number): Promise<void> {
  try {
    await (chrome as unknown as { sidePanel: { open: (opts: { tabId: number }) => Promise<void> } }).sidePanel.open({ tabId });
  } catch {
    await chrome.action.openPopup?.();
  }
}

export { TAB_TAG, openSidePanelForTab };
