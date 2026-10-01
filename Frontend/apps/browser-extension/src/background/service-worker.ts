/**
 * Service worker entrypoint — MV3 lifecycle, alarm scheduling, install/upgrade
 * hooks. Per audit finding M-17.
 */
/// <reference types="chrome" />
import { setupInstall, schedulePeriodicResync } from './install';
import { relayMessage } from './messaging';
import { startWsClient, stopWsClient } from './ws-client';
import { proxyApiRequest } from './api-client';
import { exchangeExtensionToken, refreshExtensionToken } from './auth';
import { onBadgeUpdate, setBadgeCount, clearBadge } from './badge';
import { onTabActivated } from './tabs';

chrome.runtime.onInstalled.addListener((details) => {
  void setupInstall(details);
  schedulePeriodicResync();
  void startWsClient();
});

chrome.runtime.onStartup.addListener(() => {
  schedulePeriodicResync();
  void startWsClient();
});

chrome.runtime.onMessage.addListener((msg, sender, sendResponse) => {
  relayMessage(msg, sender).then(sendResponse).catch((err: unknown) => {
    sendResponse({ ok: false, error: err instanceof Error ? err.message : 'unknown_error' });
  });
  // Return true to keep the channel open for async responses.
  return true;
});

chrome.runtime.onUserLogOut?.addListener?.((): void => {
  clearBadge();
  void stopWsClient();
});

chrome.action?.onClicked?.addListener((tab) => {
  void onTabActivated(tab);
});

// Listen for compliance-driven badge updates and side-panel opens.
chrome.runtime.onMessage.addListener((msg) => {
  if (msg?.type === 'lcc.compliance') {
    void setBadgeCount(msg.restricted ? 0 : Number(msg.count ?? 0));
  }
});

// Periodic alarms for badge sync (12h cadence).
chrome.alarms.onAlarm.addListener(async (alarm) => {
  if (alarm.name === 'lcc.badgeSync') {
    await onBadgeUpdate();
  }
});

// Token refresh on demand from popup.
chrome.runtime.onMessage.addListener((msg, _sender, sendResponse) => {
  if (msg?.type === 'lcc.refreshToken') {
    refreshExtensionToken().then((r) => sendResponse({ ok: true, ...r }))
      .catch((err: unknown) => sendResponse({ ok: false, error: err instanceof Error ? err.message : 'unknown' }));
    return true;
  }
  if (msg?.type === 'lcc.exchangeToken') {
    exchangeExtensionToken(msg.code)
      .then((r) => sendResponse({ ok: true, ...r }))
      .catch((err: unknown) => sendResponse({ ok: false, error: err instanceof Error ? err.message : 'unknown' }));
    return true;
  }
  if (msg?.type === 'lcc.apiProxy') {
    proxyApiRequest(msg.request).then(sendResponse).catch((err: unknown) => {
      sendResponse({ ok: false, error: err instanceof Error ? err.message : 'unknown' });
    });
    return true;
  }
  return undefined;
});
