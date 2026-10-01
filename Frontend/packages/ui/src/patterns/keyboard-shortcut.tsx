import * as React from 'react';
import { cn } from '../utils/cn';

export interface KeyboardShortcutProps {
  keys: string[];
  /** Platform display: "mac" shows ⌘; "win" shows Ctrl. */
  platform?: 'mac' | 'win' | 'any';
  className?: string;
  id?: string;
}

const MAC_MAP: Record<string, string> = {
  ctrl: '⌘',
  cmd: '⌘',
  meta: '⌘',
  alt: '⌥',
  option: '⌥',
  shift: '⇧',
  enter: '↵',
  escape: 'Esc',
  up: '↑',
  down: '↓',
  left: '←',
  right: '→',
};

const WIN_MAP: Record<string, string> = {
  ctrl: 'Ctrl',
  cmd: 'Win',
  meta: 'Win',
  alt: 'Alt',
  option: 'Alt',
  shift: 'Shift',
  enter: '↵',
  escape: 'Esc',
  up: '↑',
  down: '↓',
  left: '←',
  right: '→',
};

export function KeyboardShortcut({
  keys,
  platform = 'any',
  className,
  id,
}: KeyboardShortcutProps): React.ReactElement {
  return (
    <span
      id={id}
      className={cn('inline-flex items-center gap-1', className)}
      aria-label={`Keyboard shortcut: ${keys.join('+')}`}
    >
      {keys.map((k, i) => {
        const lower = k.toLowerCase();
        const map = platform === 'mac' ? MAC_MAP : platform === 'win' ? WIN_MAP : { ...MAC_MAP, ...WIN_MAP };
        const label = map[lower] ?? k.toUpperCase();
        return (
          <React.Fragment key={i}>
            {i > 0 ? <span aria-hidden="true" className="text-muted-foreground">+</span> : null}
            <kbd className="rounded border bg-muted px-1.5 py-0.5 text-xs font-mono">{label}</kbd>
          </React.Fragment>
        );
      })}
    </span>
  );
}
