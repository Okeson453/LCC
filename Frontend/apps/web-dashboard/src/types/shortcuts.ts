/**
 * Keyboard shortcut bindings.
 */

export interface ShortcutBinding {
  keys: string[];
  label: string;
  description: string;
  category: 'global' | 'composer' | 'briefing' | 'copilot';
}

export const SHORTCUTS: readonly ShortcutBinding[] = [
  { keys: ['mod', 'k'], label: '⌘K / Ctrl+K', description: 'Command palette', category: 'global' },
  { keys: ['mod', 'enter'], label: '⌘↵', description: 'Approve focused approval (Tier 2 only)', category: 'global' },
  { keys: ['mod', 'shift', 'enter'], label: '⌘⇧↵', description: 'Submit composer draft', category: 'composer' },
  { keys: ['mod', '/'], label: '⌘/', description: 'Open copilot', category: 'copilot' },
  { keys: ['g', 't'], label: 'g t', description: 'Go to Today', category: 'global' },
  { keys: ['g', 'a'], label: 'g a', description: 'Go to Approvals', category: 'global' },
  { keys: ['g', 'c'], label: 'g c', description: 'Go to Content', category: 'global' },
  { keys: ['g', 'e'], label: 'g e', description: 'Go to Engagement', category: 'global' },
  { keys: ['g', 'o'], label: 'g o', description: 'Go to Opportunities', category: 'global' },
  { keys: ['?'], label: '?', description: 'Show shortcuts', category: 'global' },
];
