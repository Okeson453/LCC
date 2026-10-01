/**
 * Copilot store — context, conversation id, panel open state.
 */

import { create } from 'zustand';

interface CopilotState {
  open: boolean;
  conversationId: string | null;
  contextModule: 'profile' | 'content' | 'engagement' | 'outreach' | 'opportunity' | 'analytics' | null;
  setOpen: (open: boolean) => void;
  setConversationId: (id: string | null) => void;
  setContextModule: (module: CopilotState['contextModule']) => void;
}

export const useCopilotStore = create<CopilotState>((set) => ({
  open: false,
  conversationId: null,
  contextModule: null,
  setOpen: (open) => set({ open }),
  setConversationId: (conversationId) => set({ conversationId }),
  setContextModule: (contextModule) => set({ contextModule }),
}));
