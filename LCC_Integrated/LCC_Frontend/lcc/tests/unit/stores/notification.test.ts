import { describe, expect, it } from 'vitest';
import { useNotificationStore } from '@/lib/stores';

describe('notification store', () => {
  it('adds and removes', () => {
    useNotificationStore.getState().clear();
    useNotificationStore.getState().push({ kind: 'info', message: 'hello' });
    expect(useNotificationStore.getState().items.length).toBe(1);
    const id = useNotificationStore.getState().items[0]!.id;
    useNotificationStore.getState().dismiss(id);
    expect(useNotificationStore.getState().items.length).toBe(0);
  });
});
