/**
 * Audit M-41 — keyboard navigation: tab order and focus rings.
 */
import { test, expect } from '@playwright/test';

test('top nav is keyboard reachable', async ({ page }) => {
  await page.goto('/today');
  await page.keyboard.press('Tab');
  const focused = await page.evaluate(() => document.activeElement?.tagName ?? '');
  expect(['A', 'BUTTON', 'INPUT', 'A', 'NAV', 'MAIN']).toContain(focused);
});

test('focus has visible outline', async ({ page }) => {
  await page.goto('/today');
  await page.keyboard.press('Tab');
  const outline = await page.evaluate(() => {
    const el = document.activeElement as HTMLElement | null;
    return el ? getComputedStyle(el).outlineStyle : null;
  });
  expect(outline).not.toBe('none');
});
