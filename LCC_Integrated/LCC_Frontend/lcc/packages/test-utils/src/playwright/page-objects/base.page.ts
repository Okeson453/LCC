/**
 * Playwright page object — base class.
 */

import { type Page, type Locator } from '@playwright/test';

export abstract class BasePage {
  constructor(protected readonly page: Page) {}

  protected byRole(role: Parameters<Page['getByRole']>[0], name?: string): Locator {
    return name ? this.page.getByRole(role, { name }) : this.page.getByRole(role);
  }

  protected byText(text: string | RegExp): Locator {
    return this.page.getByText(text);
  }

  protected byLabel(text: string): Locator {
    return this.page.getByLabel(text);
  }

  protected byTestId(testId: string): Locator {
    return this.page.getByTestId(testId);
  }

  async goto(path: string): Promise<void> {
    await this.page.goto(path);
  }

  async waitForHydration(): Promise<void> {
    await this.page.waitForLoadState('networkidle');
  }

  async screenshot(name: string): Promise<void> {
    await this.page.screenshot({ path: `screenshots/${name}.png`, fullPage: true });
  }
}
