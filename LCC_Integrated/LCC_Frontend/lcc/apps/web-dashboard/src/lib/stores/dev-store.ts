/**
 * Dev store — dev-mode toggles (mock restricted, mock low grounding, etc.).
 *
 * Active only when `NODE_ENV !== 'production'`. Used by `<DevPanel>`.
 */

import { create } from 'zustand';

interface DevState {
  mockRestricted: boolean;
  mockLowGrounding: boolean;
  mockOffline: boolean;
  setMockRestricted: (b: boolean) => void;
  setMockLowGrounding: (b: boolean) => void;
  setMockOffline: (b: boolean) => void;
  reset: () => void;
}

export const useDevStore = create<DevState>((set) => ({
  mockRestricted: false,
  mockLowGrounding: false,
  mockOffline: false,
  setMockRestricted: (mockRestricted) => set({ mockRestricted }),
  setMockLowGrounding: (mockLowGrounding) => set({ mockLowGrounding }),
  setMockOffline: (mockOffline) => set({ mockOffline }),
  reset: () =>
    set({ mockRestricted: false, mockLowGrounding: false, mockOffline: false }),
}));
