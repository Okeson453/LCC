import { test, expect } from '@playwright/test';

test('sequence detail page renders', async ({ page }) => {
  await page.goto('/outreach');
  await expect(page.getByRole('heading', { name: /outreach/i })).toBeVisible();
});
