import { useEffect } from 'react';

export type ShortcutHandler = (e: KeyboardEvent) => void;

export interface ShortcutOptions {
  /** Whether to prevent default. */
  preventDefault?: boolean;
  /** Whether to stop propagation. */
  stopPropagation?: boolean;
  /** Whether to listen on `window` (default) or a specific element ref. */
  target?: 'window' | 'document';
  /** When true, the listener is active. */
  enabled?: boolean;
}

/**
 * useKeyboardShortcut — registers a global keyboard shortcut.
 *
 * Keys is an array of key names; when all are pressed simultaneously the
 * handler fires. Examples: `['mod', 'k']`, `['ctrl', 'shift', 'p']`.
 */
export function useKeyboardShortcut(
  keys: string[],
  handler: ShortcutHandler,
  options: ShortcutOptions = {},
): void {
  const { preventDefault = true, stopPropagation = false, enabled = true } = options;

  useEffect(() => {
    if (!enabled) return;
    const target = window;
    const listener = (e: KeyboardEvent) => {
      const mod = e.metaKey || e.ctrlKey;
      const shift = e.shiftKey;
      const alt = e.altKey;
      const key = e.key.toLowerCase();

      const ok = keys.every((k) => {
        const lk = k.toLowerCase();
        if (lk === 'mod' || lk === 'cmd' || lk === 'ctrl' || lk === 'meta') return mod;
        if (lk === 'shift') return shift;
        if (lk === 'alt' || lk === 'option') return alt;
        return key === lk;
      });
      if (!ok) return;
      if (preventDefault) e.preventDefault();
      if (stopPropagation) e.stopPropagation();
      handler(e);
    };
    target.addEventListener('keydown', listener);
    return () => target.removeEventListener('keydown', listener);
  }, [keys, handler, preventDefault, stopPropagation, enabled]);
}
