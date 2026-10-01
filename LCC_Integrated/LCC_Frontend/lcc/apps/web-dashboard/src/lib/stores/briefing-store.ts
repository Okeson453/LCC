/**
 * Briefing store — last-viewed section, dismissed items.
 */

import { create } from 'zustand';

interface BriefingState {
  lastViewedSection: string | null;
  setLastViewedSection: (section: string | null) => void;
  dismissedIds: Set<string>;
  dismiss: (id: string) => void;
}

export const useBriefingStore = create<BriefingState>((set) => ({
  lastViewedSection: null,
  setLastViewedSection: (lastViewedSection) => set({ lastViewedSection }),
  dismissedIds: new Set(),
  dismiss: (id) =>
    set((s) => {
      const next = new Set(s.dismissedIds);
      next.add(id);
      return { dismissedIds: next };
    }),
}));
