/**
 * Notification store — unread count + seen ids.
 *
 * Real notifications arrive via WS push; this store keeps UI-only state.
 */

import { create } from 'zustand';

interface NotificationState {
  unreadCount: number;
  seenIds: Set<string>;
  setUnreadCount: (n: number) => void;
  markSeen: (id: string) => void;
  reset: () => void;
}

export const useNotificationStore = create<NotificationState>((set) => ({
  unreadCount: 0,
  seenIds: new Set(),
  setUnreadCount: (unreadCount) => set({ unreadCount }),
  markSeen: (id) =>
    set((s) => {
      const next = new Set(s.seenIds);
      next.add(id);
      return { seenIds: next, unreadCount: Math.max(0, s.unreadCount - 1) };
    }),
  reset: () => set({ unreadCount: 0, seenIds: new Set() }),
}));
