import { type Page } from '@playwright/test';
import { BasePage } from './base.page';

export class TodayPage extends BasePage {
  constructor(page: Page) {
    super(page);
  }

  async goto(): Promise<void> {
    await this.page.goto('/today');
  }

  async getApprovalCount(): Promise<number> {
    return this.page.locator('[data-testid="approval-item"]').count();
  }

  async clickFirstApproval(): Promise<void> {
    await this.page.locator('[data-testid="approval-item"]').first().click();
  }

  async expectBriefingVisible(): Promise<void> {
    await this.page.getByText(/approvals due/i).first().waitFor();
  }
}
