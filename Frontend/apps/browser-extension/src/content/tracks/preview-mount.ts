/**
 * Track B · ActionPreview mount. Tier-1 only.
 */
import { createRoot, type Root } from 'react-dom/client';
import { ActionPreview } from '../components/ConfirmPrompt';

const MOUNT_ATTR = 'data-lcc-preview';

export interface ActionPreviewAction {
  trackId: string;
  tier: 1 | 2 | 3 | 4 | 5;
  message: string;
  triggerLabel: string;
  onTrigger: (trackId: string, tier: 1 | 2 | 3 | 4 | 5) => void;
}

const roots = new WeakMap<HTMLElement, Root>();

export function mountActionPreview(target: HTMLElement, action: Omit<ActionPreviewAction, 'onTrigger'>): void {
  if (target.hasAttribute(MOUNT_ATTR)) return;
  target.setAttribute(MOUNT_ATTR, '1');
  const host = document.createElement('div');
  target.appendChild(host);

  const root = createRoot(host);
  roots.set(target, root);
  root.render(
    <ActionPreview
      trackId={action.trackId}
      tier={action.tier}
      message={action.message}
      triggerLabel={action.triggerLabel}
      onTrigger={(id, tier) => chrome.runtime.sendMessage({ from: 'content', type: 'lcc.actionClicked', trackId: id, tier })}
    />,
  );
}
