/**
 * ActionPreview + ConfirmPrompt React components for in-page overlay.
 * Tier 1 only — never performs a DOM mutation on LinkedIn.
 * Audit ref: M-25.
 */
import * as React from 'react';

export interface ActionPreviewProps {
  trackId: string;
  tier: 1 | 2 | 3 | 4 | 5;
  message: string;
  triggerLabel: string;
  onTrigger: (trackId: string, tier: 1 | 2 | 3 | 4 | 5) => void;
}

export function ActionPreview({ trackId, tier, message, triggerLabel, onTrigger }: ActionPreviewProps): React.ReactElement {
  return (
    <div role="region" aria-label="LCC action preview"
      style={{ background: 'rgba(10,102,194,0.06)', border: '1px solid rgba(10,102,194,0.4)', padding: 8, borderRadius: 8, fontFamily: 'Inter, system-ui, sans-serif', fontSize: 12 }}>
      <p style={{ margin: 0 }}>{message}</p>
      <p style={{ margin: 4, fontSize: 10, opacity: 0.6 }}>Tier {tier}</p>
      <button
        type="button"
        onClick={() => onTrigger(trackId, tier)}
        style={{ padding: '4px 8px', background: '#0a66c2', color: '#fff', border: 0, borderRadius: 6, cursor: 'pointer' }}
      >
        {triggerLabel}
      </button>
    </div>
  );
}

export interface ConfirmPromptProps {
  open: boolean;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  onConfirm: () => void;
  onCancel: () => void;
}

export function ConfirmPrompt({ open, message, confirmLabel = 'Confirm', cancelLabel = 'Cancel', onConfirm, onCancel }: ConfirmPromptProps): React.ReactElement | null {
  if (!open) return null;
  return (
    <div role="dialog" aria-modal="true" aria-label="LCC confirm"
      style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,0.4)', display: 'grid', placeItems: 'center', zIndex: 2147483647 }}>
      <div style={{ background: '#fff', padding: 16, borderRadius: 8, maxWidth: 320, fontFamily: 'Inter, system-ui, sans-serif', color: '#0a0a0a' }}>
        <p style={{ marginTop: 0 }}>{message}</p>
        <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end' }}>
          <button type="button" onClick={onCancel}>{cancelLabel}</button>
          <button type="button" onClick={onConfirm} style={{ background: '#0a66c2', color: '#fff', border: 0, padding: '6px 10px', borderRadius: 6 }}>{confirmLabel}</button>
        </div>
      </div>
    </div>
  );
}

export interface TrackBadgeProps {
  trackId: string;
  tier: 1 | 2 | 3 | 4 | 5;
}

export function TrackBadge({ trackId, tier }: TrackBadgeProps): React.ReactElement {
  return <span style={{ padding: '2px 6px', background: '#0a66c2', color: '#fff', borderRadius: 4, fontSize: 10 }}>LCC · {trackId} · t{tier}</span>;
}
