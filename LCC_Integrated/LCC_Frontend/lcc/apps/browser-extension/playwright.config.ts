import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './test/e2e',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  workers: 1,
  reporter: process.env.CI ? [['github']] : 'list',
  use: {
    baseURL: process.env.E2E_BASE_URL ?? 'http://localhost:8080',
    trace: 'on-first-retry',
  },
  projects: [
    { name: 'chromium', use: { channel: 'chromium' } },
  ],
  webServer: {
    command: 'pnpm build:dev && node test/e2e/extension-server.cjs',
    port: 8082,
    reuseExistingServer: true,
    timeout: 120_000,
  },
});
