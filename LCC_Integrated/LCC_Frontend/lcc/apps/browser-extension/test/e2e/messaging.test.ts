import { test, expect } from '@playwright/test';
import { MSG, isMessage } from '@/lib/messaging';

test('messaging constants match', () => {
  expect(MSG.openSidePanel).toBe('lcc.openSidePanel');
});

test('isMessage narrows correctly', () => {
  expect(isMessage({ type: MSG.openSidePanel }, MSG.openSidePanel)).toBe(true);
  expect(isMessage({ type: 'lcc.unknown' }, MSG.openSidePanel)).toBe(false);
});
