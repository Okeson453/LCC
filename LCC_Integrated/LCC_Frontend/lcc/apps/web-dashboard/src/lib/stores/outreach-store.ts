/**
 * Outreach store — sequence builder in-progress state.
 */

import { create } from 'zustand';

interface OutreachState {
  name: string;
  selectedContactIds: string[];
  templateId: string | null;
  startAt: string | null;
  setName: (name: string) => void;
  setSelectedContactIds: (ids: string[]) => void;
  setTemplateId: (id: string | null) => void;
  setStartAt: (at: string | null) => void;
  reset: () => void;
}

export const useOutreachStore = create<OutreachState>((set) => ({
  name: '',
  selectedContactIds: [],
  templateId: null,
  startAt: null,
  setName: (name) => set({ name }),
  setSelectedContactIds: (selectedContactIds) => set({ selectedContactIds }),
  setTemplateId: (templateId) => set({ templateId }),
  setStartAt: (startAt) => set({ startAt }),
  reset: () =>
    set({ name: '', selectedContactIds: [], templateId: null, startAt: null }),
}));
