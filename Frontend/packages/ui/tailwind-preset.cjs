/**
 * Tailwind preset for @lcc/ui.
 * Re-exports the @lcc/tokens preset and adds shadcn/ui utility classes.
 */

module.exports = {
  presets: [require('@lcc/tokens/tailwind-preset.cjs')],
  darkMode: ['class', '[data-theme="dark"]'],
  content: [
    './src/**/*.{ts,tsx}',
    './node_modules/@lcc/ui/src/**/*.{ts,tsx}',
  ],
  theme: {
    extend: {},
  },
  plugins: [],
};
