import { test, expect } from '@playwright/test';
import AxeRunner from '@axe-core/playwright';

test('approval dialog traps focus', async ({ page }) => {
  await page.goto('/approvals');
  const dialog = page.getByRole('dialog').first();
  if ((await dialog.count()) === 0) {
    test.skip(true, 'no approval dialog present in this environment');
  }
  await dialog.waitFor({ state: 'visible' });

  const traps = async (): Promise<boolean> => {
    const firstFocusable = page.locator('[role="dialog"] button, [role="dialog"] [tabindex]:not([tabindex="-1"])').first();
    const last = page.locator('[role="dialog"] button, [role="dialog"] [tabindex]:not([tabindex="-1"])').last();
    await firstFocusable.focus();
    for (let i = 0; i < 30; i += 1) await page.keyboard.press('Tab');
    return await page.evaluate(() => {
      const el = document.activeElement as HTMLElement | null;
      return Boolean(el && el.closest('[role="dialog"]'));
    });
  };
  expect(await traps()).toBe(true);

  const ax = await new AxeRunner({ page } as unknown as { page: typeof page }).include('[role="dialog"]').analyze();
  expect(ax.violations.filter((v: { impact?: string }) => v.impact === 'critical')).toHaveLength(0);
});
