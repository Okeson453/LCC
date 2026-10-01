/**
 * Badge management — surfaces pending approval count and toggles color.
 */
export async function setBadgeCount(count: number): Promise<void> {
  await chrome.action.setBadgeText({ text: count > 0 ? String(count) : '' });
  await chrome.action.setBadgeBackgroundColor({ color: '#0a66c2' });
}

export async function clearBadge(): Promise<void> {
  await chrome.action.setBadgeText({ text: '' });
}

export async function onBadgeUpdate(): Promise<void> {
  // Lazy import to avoid circular deps during module init.
  const { proxyApiRequest } = await import('./api-client');
  const res = await proxyApiRequest({ method: 'GET', path: '/api/v1/approvals/queue?status=pending' });
  if (!res.ok || res.data === undefined || res.data === null) return;
  const data = res.data as { items?: unknown[] };
  await setBadgeCount(data.items?.length ?? 0);
}
