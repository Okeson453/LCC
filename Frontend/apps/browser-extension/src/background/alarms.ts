const DAILY_ALARM = 'lcc.daily';

export function scheduleDailyAlarm(): void {
  chrome.alarms.create(DAILY_ALARM, { periodInMinutes: 60 * 12 }); // twice a day.
}

export async function onAlarm(alarm: chrome.alarms.Alarm): Promise<void> {
  if (alarm.name === DAILY_ALARM) {
    await refetchApprovalCount();
  }
}

async function refetchApprovalCount(): Promise<void> {
  try {
    const cfg = await chrome.storage.local.get(['apiBase']);
    if (!cfg.apiBase) return;
    const token = (await chrome.storage.local.get(['accessToken'])).accessToken as string | undefined;
    if (!token) return;

    const res = await fetch(`${cfg.apiBase}/approvals/queue?status=pending`, {
      headers: { Authorization: `Bearer ${token}`, 'x-trace-id': crypto.randomUUID() },
    });
    if (!res.ok) return;
    const json = (await res.json()) as { items?: unknown[] };
    const count = json.items?.length ?? 0;
    await chrome.action.setBadgeText({ text: count > 0 ? String(count) : '' });
  } catch {
    /* ignore */
  }
}
