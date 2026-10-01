import { type Page } from '@playwright/test';
import { BasePage } from './base.page';

export class ApprovalDialogPage extends BasePage {
  constructor(page: Page) {
    super(page);
  }

  async expectOpen(): Promise<void> {
    await this.page.getByRole('dialog').waitFor();
  }

  async expectKbCitations(): Promise<void> {
    const citations = await this.page.getByLabel(/kb grounding citations/i).count();
    if (citations === 0) throw new Error('No KB citations rendered');
  }

  async expectRiskTierBadge(tier: number): Promise<void> {
    await this.page.getByLabel(new RegExp(`risk tier ${tier} of 5`, 'i')).waitFor();
  }

  async editMessage(text: string): Promise<void> {
    const ta = this.page.getByLabel(/editable preview/i);
    await ta.fill(text);
  }

  async approve(): Promise<void> {
    await this.page.getByRole('button', { name: /approve/i }).click();
  }

  async reject(): Promise<void> {
    await this.page.getByRole('button', { name: /reject/i }).click();
  }

  async close(): Promise<void> {
    await this.page.keyboard.press('Escape');
  }
}
