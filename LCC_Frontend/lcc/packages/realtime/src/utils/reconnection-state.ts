/**
 * Reconnection state — exposed to UI for transparent indicators.
 *
 * The `RealtimeClient` keeps an instance of this state object; consumers
 * subscribe via `useRealtimeStatus()`.
 */

export type RealtimeTransport = 'ws' | 'sse' | 'polling' | 'none';

export interface RealtimeStatus {
  transport: RealtimeTransport;
  status: 'connecting' | 'connected' | 'reconnecting' | 'degraded' | 'disconnected';
  attempt: number;
  lastConnectedAt: number | null;
  lastDisconnectedAt: number | null;
  reason: string | null;
}

export class ReconnectionState {
  private _status: RealtimeStatus = {
    transport: 'none',
    status: 'disconnected',
    attempt: 0,
    lastConnectedAt: null,
    lastDisconnectedAt: null,
    reason: null,
  };

  private listeners = new Set<(s: RealtimeStatus) => void>();

  get current(): RealtimeStatus {
    return { ...this._status };
  }

  update(patch: Partial<RealtimeStatus>): void {
    this._status = { ...this._status, ...patch };
    for (const l of this.listeners) l(this.current);
  }

  subscribe(listener: (s: RealtimeStatus) => void): () => void {
    this.listeners.add(listener);
    listener(this.current);
    return () => {
      this.listeners.delete(listener);
    };
  }
}
