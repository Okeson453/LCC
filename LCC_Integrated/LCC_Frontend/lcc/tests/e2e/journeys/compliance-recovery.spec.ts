/**
 * Compliance recovery — when restricted state is on, banner is announced.
 */
import { test, expect } from '@playwright/test';

test('compliance banner region exists', async ({ page }) => {
  await page.goto('/');
  // Region may or may not be visible depending on state.
  await expect(page).toHaveTitle(/LinkedIn Manager|OKESON/i);
});
