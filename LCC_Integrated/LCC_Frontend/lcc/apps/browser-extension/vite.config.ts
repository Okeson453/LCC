import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { resolve } from 'node:path';
import { readFileSync } from 'node:fs';

const pkg = JSON.parse(readFileSync(new URL('./package.json', import.meta.url), 'utf-8')) as { name: string };

export default defineConfig({
  plugins: [react()],
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
  build: {
    target: 'es2022',
    sourcemap: true,
    rollupOptions: {
      input: {
        popup: 'src/popup/main.tsx',
        sidepanel: 'src/sidepanel/main.tsx',
        background: 'src/background/index.ts',
        content: 'src/content/index.ts',
      },
      output: {
        entryFileNames: (chunk) => {
          if (chunk.name === 'background') return 'background/service-worker.js';
          if (chunk.name === 'content') return 'content/content-script.js';
          return `${chunk.name}/index.js`;
        },
        assetFileNames: 'assets/[name].[ext]',
      },
    },
    cssCodeSplit: true,
  },
  test: {
    globals: true,
    environment: 'happy-dom',
    setupFiles: ['./test/setup.ts'],
    include: ['src/**/*.{test,spec}.{ts,tsx}'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html'],
      thresholds: { lines: 70, statements: 70, branches: 65, functions: 70 },
    },
  },
});
