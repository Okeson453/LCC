import type { Config } from 'tailwindcss';
import uiPreset from '@lcc/ui/tailwind-preset';

const config: Config = {
  presets: [uiPreset],
  darkMode: ['class', '[data-theme="dark"]'],
  content: [
    './src/**/*.{ts,tsx}',
    '../../packages/ui/src/**/*.{ts,tsx}',
    '../../packages/approval-gate/src/**/*.{ts,tsx}',
    '../../packages/compliance-state/src/**/*.{ts,tsx}',
  ],
  theme: {
    extend: {},
  },
  plugins: [],
};

export default config;
