/**
 * RestrictedStateBlocker — surfaces non-negotiable §5 (governor restricted
 * state disables every Tier 2+ action on the page).
 * Audit ref: M-25 + A-03.
 */
import * as React from 'react';

export interface RestrictedStateBlockerProps {
  reason: string;
  until: string | null;
  onOpenDashboard: () => void;
}

export function RestrictedStateBlocker({ reason, until, onOpenDashboard }: RestrictedStateBlockerProps): React.ReactElement {
  return (
    <div role="alert" style={{ background: '#fff4e5', border: '1px solid #d97706', padding: 12, borderRadius: 8, fontFamily: 'Inter, system-ui, sans-serif', fontSize: 12 }}>
      <strong>Account restricted.</strong>
      <p style={{ margin: '4px 0' }}>Reason: {reason}.</p>
      {until ? <p style={{ margin: '4px 0' }}>Until: {new Date(until).toLocaleString()}</p> : null}
      <button type="button" onClick={onOpenDashboard} style={{ padding: '4px 8px', background: '#0a66c2', color: '#fff', border: 0, borderRadius: 6 }}>Open dashboard</button>
    </div>
  );
}
