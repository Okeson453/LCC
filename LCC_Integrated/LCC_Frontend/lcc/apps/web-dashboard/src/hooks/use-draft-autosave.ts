/**
 * useDraftAutosave — local autosave for the composer / sequence builder.
 *
 * Note: this is UI-only state (localStorage). The backend persists drafts
 * via the createContent / createSequence endpoints.
 */

import { useEffect, useRef } from 'react';

export interface UseDraftAutosaveOptions<T> {
  /** LocalStorage key. */
  key: string;
  /** Current state. */
  state: T;
  /** Debounce interval (ms). Defaults to 1000. */
  intervalMs?: number;
  /** Optional serializer. */
  serialize?: (state: T) => string;
}

export function useDraftAutosave<T>({
  key,
  state,
  intervalMs = 1000,
  serialize = JSON.stringify,
}: UseDraftAutosaveOptions<T>): void {
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    if (timer.current) clearTimeout(timer.current);
    timer.current = setTimeout(() => {
      try {
        if (typeof window !== 'undefined') {
          window.localStorage.setItem(key, serialize(state));
        }
      } catch {
        // ignore quota errors
      }
    }, intervalMs);
    return () => {
      if (timer.current) clearTimeout(timer.current);
    };
  }, [key, state, intervalMs, serialize]);
}

export function loadDraft<T>(key: string, fallback: T): T {
  if (typeof window === 'undefined') return fallback;
  try {
    const raw = window.localStorage.getItem(key);
    if (!raw) return fallback;
    return JSON.parse(raw) as T;
  } catch {
    return fallback;
  }
}

export function clearDraft(key: string): void {
  if (typeof window === 'undefined') return;
  try {
    window.localStorage.removeItem(key);
  } catch {
    // ignore
  }
}
