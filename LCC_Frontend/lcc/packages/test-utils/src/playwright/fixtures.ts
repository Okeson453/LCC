/**
 * Playwright fixtures — auth state, page objects.
 */

import { test as base } from '@playwright/test';
import { TodayPage } from './page-objects/today.page';
import { ComposerPage } from './page-objects/composer.page';
import { ApprovalDialogPage } from './page-objects/approval.dialog';

export const test = base.extend<{
  todayPage: TodayPage;
  composerPage: ComposerPage;
  approvalDialog: ApprovalDialogPage;
}>({
  todayPage: async ({ page }, use) => {
    await use(new TodayPage(page));
  },
  composerPage: async ({ page }, use) => {
    await use(new ComposerPage(page));
  },
  approvalDialog: async ({ page }, use) => {
    await use(new ApprovalDialogPage(page));
  },
});

export { expect } from '@playwright/test';
