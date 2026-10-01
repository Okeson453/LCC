/**
 * Sidepanel hooks — start with restricted-state watcher.
 * Audit ref: M-34.
 */
import * as React from 'react';
import { startRestrictedWatcher } from '../../lib/compliance';

export function useSidepanelCompliance(memberId: string): { restricted: boolean; reason: string | null; until: string | null } {
  const [state, setState] = React.useState({ restricted: false, reason: null as string | null, until: null as string | null });
  React.useEffect(() => {
    const w = startRestrictedWatcher(memberId, setState);
    return () => w.stop();
  }, [memberId]);
  return state;
}
