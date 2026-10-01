import { describe, expect, it } from 'vitest';
import { useSidebarStore } from '@/lib/stores';

describe('sidebar store', () => {
  it('toggles collapsed', () => {
    useSidebarStore.getState().setCollapsed(false);
    expect(useSidebarStore.getState().collapsed).toBe(false);
    useSidebarStore.getState().toggle();
    expect(useSidebarStore.getState().collapsed).toBe(true);
  });
});
