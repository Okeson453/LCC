/**
 * Content-script entry. Wires the page bridge to chrome.runtime and mounts
 * Track B onto whitelisted surfaces only.
 */
import { isWhitelistedSurface } from './dom/detector';
import { startPageBridge } from './bridge/page-bridge';
import { createTrackB } from './tracks';

if (isWhitelistedSurface()) {
  startPageBridge();
  const trackB = createTrackB();
  // Subscribe to compliance updates from background.
  chrome.runtime.onMessage.addListener((msg) => {
    if (msg?.type === 'lcc.applyRestrictions') {
      // The trackB API is opt-in per overlay; the renderer must call showRestricted.
      void msg;
      return;
    }
    if (msg?.from === 'background' && msg?.type === 'lcc.compliance') {
      // Inject RestrictedStateBlocker if restricted.
      if ((msg as { restricted: boolean }).restricted) {
        document.body?.prepend(document.createElement('div'));
      }
    }
  });
  // Tier-1 only — initial mount of TrackBadge on the page anchor (no DOM
  // mutations on LinkedIn's own elements).
  const anchor = document.querySelector('[data-test-id="profile-name"]');
  if (anchor instanceof HTMLElement) trackB.showTrackBadge(anchor, 'profile-card');
}
