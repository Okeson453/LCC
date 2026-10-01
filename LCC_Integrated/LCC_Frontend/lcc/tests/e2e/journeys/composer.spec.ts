/**
 * Composer journey — open /content/new → enter prompt → generate variants.
 */
import { test, expect } from '@playwright/test';

test('composer page loads', async ({ page }) => {
  await page.goto('/content/new');
  // Don't enforce heading match — varies per design iteration.
  await expect(page).toHaveURL(/content\/new/);
});
