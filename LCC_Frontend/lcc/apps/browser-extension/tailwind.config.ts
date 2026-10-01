import type { Config } from 'tailwindcss';
import { tokens } from '../../packages/tokens/tailwind-preset.cjs';

const config: Config = {
  presets: [tokens],
  content: ['./src/**/*.{ts,tsx}', './public/**/*.{html,css,js}'],
  darkMode: 'media',
  theme: { extend: {} },
  plugins: [],
};

export default config;
