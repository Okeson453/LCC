import { type Page } from '@playwright/test';
import { BasePage } from './base.page';

export class ComposerPage extends BasePage {
  constructor(page: Page) {
    super(page);
  }

  async goto(): Promise<void> {
    await this.page.goto('/content/new');
  }

  async fillPrompt(prompt: string): Promise<void> {
    await this.page.getByLabel(/prompt/i).fill(prompt);
  }

  async selectVariant(name: string): Promise<void> {
    await this.page.getByRole('button', { name }).click();
  }

  async submitForApproval(): Promise<void> {
    await this.page.getByRole('button', { name: /submit for approval/i }).click();
  }

  async expectPreviewVisible(): Promise<void> {
    await this.page.locator('[data-testid="variant-preview"]').first().waitFor();
  }
}
