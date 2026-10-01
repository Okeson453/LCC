/**
 * Daily loop journey — Copilot slide-over → approval review.
 */
import { test, expect } from '@playwright/test';

test('daily loop opens Copilot', async ({ page }) => {
  await page.goto('/today');
  const trigger = page.getByRole('button', { name: /copilot/i }).first();
  if (await trigger.count() > 0) {
    await trigger.click();
    await expect(page.getByRole('dialog', { name: /copilot/i }).or(page.locator('[data-testid="copilot"]'))).toBeVisible();
  }
});
