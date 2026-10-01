/**
 * Audit M-42 — visual regression snapshots. Thresholds intentionally lenient
 * in dev; tight in CI.
 */
import { test, expect } from '@playwright/test';

test.describe('visual snapshots', () => {
  test('today page', async ({ page }) => {
    await page.goto('/today');
    await page.waitForLoadState('networkidle');
    await expect(page).toHaveScreenshot('today.png', { maxDiffPixelRatio: 0.05 });
  });

  test('opportunities board', async ({ page }) => {
    await page.goto('/opportunities');
    await page.waitForLoadState('networkidle');
    await expect(page).toHaveScreenshot('opportunities.png', { maxDiffPixelRatio: 0.05 });
  });

  test('approvals queue', async ({ page }) => {
    await page.goto('/approvals');
    await page.waitForLoadState('networkidle');
    await expect(page).toHaveScreenshot('approvals.png', { maxDiffPixelRatio: 0.05 });
  });
});
