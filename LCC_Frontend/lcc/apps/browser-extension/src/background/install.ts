/**
 * First-run setup. Audit ref: install.ts was already present but renamed to
 * stay consistent with the new background/ module.
 */
const DAILY_ALARM = 'lcc.badgeSync';

export async function setupInstall(_details: chrome.runtime.InstalledDetails): Promise<void> {
  const existing = await chrome.storage.local.get(['cfg']);
  if (existing.cfg) return;
  await chrome.storage.local.set({
    cfg: { apiBase: 'http://localhost:8080', popMode: 'panel', redactBeforeLog: true },
    onboarded: true,
  });
}

export function schedulePeriodicResync(): void {
  chrome.alarms.create(DAILY_ALARM, { periodInMinutes: 60 * 12 });
}

export async function onAlarm(alarm: chrome.alarms.Alarm): Promise<void> {
  if (alarm.name === DAILY_ALARM) {
    const { onBadgeUpdate } = await import('./badge');
    await onBadgeUpdate();
  }
}
