/**
 * Background-side WebSocket client. Connects to the gateway's realtime
 * channel and pushes events to the popup / sidepanel via chrome.runtime.
 * Audit ref: M-20 + A-16 (extension ↔ backend WS).
 */
import { getAccessToken, getConfig } from '../lib/storage';
import { computeBackoffMs } from '@lcc/realtime';
import type { BackgroundMessage } from '../types';

let socket: WebSocket | null = null;
let retries = 0;
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
let pingTimer: ReturnType<typeof setInterval> | null = null;

export async function startWsClient(): Promise<void> {
  if (socket && socket.readyState <= 1) return;
  await connect();
}

export async function stopWsClient(): Promise<void> {
  if (reconnectTimer) { clearTimeout(reconnectTimer); reconnectTimer = null; }
  if (pingTimer) { clearInterval(pingTimer); pingTimer = null; }
  if (socket) { socket.close(); socket = null; }
}

async function connect(): Promise<void> {
  const cfg = await getConfig();
  const token = await getAccessToken();
  if (!token) return;
  const url = cfg.apiBase.replace(/^http/, 'ws') + '/ws';
  try {
    socket = new WebSocket(url, ['bearer', token]);
    socket.addEventListener('open', () => {
      retries = 0;
      if (pingTimer) clearInterval(pingTimer);
      pingTimer = setInterval(() => socket?.send(JSON.stringify({ kind: 'ping' })), 20_000);
    });
    socket.addEventListener('message', (ev) => {
      try {
        const parsed = JSON.parse(String(ev.data)) as { kind?: string };
        if (parsed.kind === 'compliance') {
          broadcast({ from: 'background', type: 'lcc.compliance', restricted: Boolean((parsed as { restricted?: boolean }).restricted), reason: (parsed as { reason?: string | null }).reason ?? null });
        } else if (parsed.kind === 'badge') {
          broadcast({ from: 'background', type: 'lcc.badgeUpdate', count: Number((parsed as { count?: number }).count ?? 0) });
        }
      } catch {
        /* ignore non-JSON */
      }
    });
    socket.addEventListener('close', () => scheduleReconnect());
    socket.addEventListener('error', () => socket?.close());
  } catch {
    scheduleReconnect();
  }
}

function scheduleReconnect(): void {
  if (reconnectTimer) return;
  const delay = computeBackoffMs(retries, 1_000, 30_000, 250);
  retries += 1;
  reconnectTimer = setTimeout(() => {
    reconnectTimer = null;
    void connect();
  }, delay);
}

function broadcast(msg: BackgroundMessage): void {
  chrome.runtime.sendMessage(msg).catch(() => {
    /* no listener attached — fine during idle */
  });
}
