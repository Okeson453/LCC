import { test, expect } from '@playwright/test';

test('opportunity board renders columns', async ({ page }) => {
  await page.goto('/opportunities');
  await expect(page.getByRole('heading', { name: /opportunities/i })).toBeVisible();
});
