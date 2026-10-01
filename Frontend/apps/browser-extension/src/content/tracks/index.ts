/**
 * Track B action implementations — Tier 1 read-only overlays ONLY.
 * Audit ref: M-27 + A-04.
 *
 * All mutations route through chrome.runtime → background → gateway.
 * The extension NEVER writes to the LinkedIn DOM directly.
 */
import { mountActionPreview, type ActionPreviewAction } from './preview-mount';
import { mountConfirmPrompt } from './confirm-mount';
import { mountTrackBadge } from './badge-mount';

export interface TrackBApi {
  showActionPreview(target: HTMLElement, action: Omit<ActionPreviewAction, 'onTrigger'>): void;
  showConfirmPrompt(target: HTMLElement, message: string): void;
  showTrackBadge(target: HTMLElement, trackId: string): void;
}

export function createTrackB(): TrackBApi {
  return {
    showActionPreview: mountActionPreview,
    showConfirmPrompt: mountConfirmPrompt,
    showTrackBadge: mountTrackBadge,
  };
}
