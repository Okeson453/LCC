/**
 * Track B · TrackBadge mount.
 */
import { createRoot, type Root } from 'react-dom/client';
import { TrackBadge } from '../components/ConfirmPrompt';

export function mountTrackBadge(target: HTMLElement, trackId: string): void {
  const host = document.createElement('span');
  host.style.cssText = 'margin-left: 8px; display: inline-flex; align-items: center;';
  target.appendChild(host);
  const root: Root = createRoot(host);
  root.render(<TrackBadge trackId={trackId} tier={1} />);
}
