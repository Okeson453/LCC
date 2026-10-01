import { defineConfig } from 'vitest/config';
import { resolve } from 'node:path';

export default defineConfig({
  test: {
    globals: true,
    environment: 'happy-dom',
    setupFiles: ['./test/setup.ts'],
    include: [
      'src/**/*.{test,spec}.{ts,tsx}',
      'test/unit/**/*.{test,spec}.{ts,tsx}',
      'test/component/**/*.{test,spec}.{ts,tsx}',
    ],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html', 'lcov'],
      include: ['src/**/*.{ts,tsx}'],
      exclude: ['src/**/*.{test,spec}.{ts,tsx}', 'src/**/*.d.ts'],
      thresholds: { lines: 70, statements: 70, branches: 65, functions: 70 },
    },
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, './src'),
      '@lcc/tokens': resolve(__dirname, '../../packages/tokens/src/index.ts'),
      '@lcc/ui': resolve(__dirname, '../../packages/ui/src/index.ts'),
      '@lcc/api-types': resolve(__dirname, '../../packages/api-types/src/index.ts'),
      '@lcc/approval-gate': resolve(__dirname, '../../packages/approval-gate/src/index.ts'),
      '@lcc/compliance-state': resolve(__dirname, '../../packages/compliance-state/src/index.ts'),
      '@lcc/i18n': resolve(__dirname, '../../packages/i18n/src/index.ts'),
      '@lcc/realtime': resolve(__dirname, '../../packages/realtime/src/index.ts'),
      '@lcc/test-utils': resolve(__dirname, '../../packages/test-utils/src/index.ts'),
    },
  },
});
