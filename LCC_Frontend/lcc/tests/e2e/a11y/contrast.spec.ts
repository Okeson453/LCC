import { test, expect } from '@playwright/test';
import AxeRunner from '@axe-core/playwright';

test('today page passes axe (no critical)', async ({ page }) => {
  await page.goto('/today');
  const res = await new AxeRunner({ page } as unknown as { page: typeof page }).analyze();
  const critical = res.violations.filter((v: { impact?: string }) => v.impact === 'critical');
  expect(critical, JSON.stringify(critical, null, 2)).toHaveLength(0);
});
