'use client';

/**
 * AuthBridge — handles extension ↔ web dashboard auth exchange.
 * Audit ref: M-26 + A-15.
 */
import * as React from 'react';
import { exchangeExtensionToken } from '../../background/auth';

export interface AuthBridgeProps {
  onAuthed: (memberId: string) => void;
}

export function AuthBridge({ onAuthed }: AuthBridgeProps): React.ReactElement {
  React.useEffect(() => {
    void doExchange(onAuthed);
  }, [onAuthed]);
  return <p style={{ fontFamily: 'Inter, system-ui, sans-serif', fontSize: 12 }}>Authenticating…</p>;
}

async function doExchange(onAuthed: (memberId: string) => void): Promise<void> {
  try {
    const params = new URLSearchParams(location.search);
    const code = params.get('code');
    if (!code) return;
    const bundle = await exchangeExtensionToken(code);
    onAuthed(bundle.member_id);
  } catch {
    /* swallow — popup shows error */
  }
}
