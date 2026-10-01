/**
 * Restricted-state watcher — polls the gateway's `/admin/compliance/restrictions/{memberId}`
 * endpoint while the extension is open. Audit ref: M-29 + A-03.
 */
import { getAccessToken, getConfig } from '../storage';

export interface RestrictedState {
  restricted: boolean;
  reason: string | null;
  until: string | null;
}

const POLL_MS = 60_000;

export type Watcher = { stop: () => void };

export function startRestrictedWatcher(memberId: string, onUpdate: (s: RestrictedState) => void): Watcher {
  let stopped = false;
  let timer: ReturnType<typeof setInterval> | null = null;

  void fetchOnce();
  timer = setInterval(() => { void fetchOnce(); }, POLL_MS);

  return {
    stop: (): void => {
      stopped = true;
      if (timer) clearInterval(timer);
    },
  };

  async function fetchOnce(): Promise<void> {
    if (stopped) return;
    const cfg = await getConfig();
    const token = await getAccessToken();
    if (!token) return;
    try {
      const res = await fetch(`${cfg.apiBase}/api/v1/admin/compliance/restrictions/${memberId}`, {
        headers: { Authorization: `Bearer ${token}`, 'x-trace-id': crypto.randomUUID() },
      });
      if (!res.ok) return;
      const body = (await res.json()) as RestrictedState;
      onUpdate(body);
    } catch {
      /* network blip — keep last-known state */
    }
  }
}
