/**
 * Extension integration test — non-negotiable §5: restricted state disables
 * all actions in the popup. Audit ref: M-47.
 */
import { test, expect } from '@playwright/test';

test('popup shows restricted banner when gateway returns restricted=true', async ({ page }) => {
  await page.route('**/api/v1/admin/compliance/restrictions/me', (r) =>
    r.fulfill({ json: { restricted: true, reason: 'cooldown', until: null } }),
  );
  await page.route('**/api/v1/approvals/queue*', (r) => r.fulfill({ json: { items: [] } }));

  await page.goto('chrome-extension://__test__/sidepanel/index.html');
  await page.getByText(/Compliance/i).click({ timeout: 5_000 });
  await expect(page.getByText(/Restricted/i)).toBeVisible();
});
