/**
 * Page ↔ extension bridge. The content script surfaces observations as
 * `lcc.detectedSurface` events; the background relays to the popup.
 * Audit ref: M-24.
 */
import { detectSurface, type LinkedInSurface } from '../dom/detector';
import type { ContentMessage } from '../../types';

export function startPageBridge(): void {
  let lastEmitted: string | null = null;

  const tick = (): void => {
    const surface = detectSurface();
    const key = JSON.stringify(surface);
    if (key === lastEmitted) return;
    lastEmitted = key;
    const msg: ContentMessage = surface === null
      ? { from: 'content', type: 'lcc.detectedSurface', surface: 'feed' }
      : { from: 'content', type: 'lcc.detectedSurface', surface: surface.kind };
    void chrome.runtime.sendMessage(msg).catch(() => {/* no listener */});
  };

  tick();
  document.addEventListener('visibilitychange', tick);
  window.addEventListener('popstate', tick, { passive: true });

  // Hook MutationObserver to detect route changes (LinkedIn is SPA).
  const mo = new MutationObserver(() => tick());
  mo.observe(document.body, { childList: true, subtree: false });
}
