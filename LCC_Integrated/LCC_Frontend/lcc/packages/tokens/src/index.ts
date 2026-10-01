/**
 * @lcc/tokens — public exports.
 *
 * Single source of truth for design tokens across all frontend surfaces
 * (Web Dashboard, Mobile PWA, Browser Extension).
 */

// CSS — import via global stylesheet bundler
export const cssFiles = [
  '@lcc/tokens/css/colors.css',
  '@lcc/tokens/css/spacing.css',
  '@lcc/tokens/css/typography.css',
  '@lcc/tokens/css/radius.css',
  '@lcc/tokens/css/shadow.css',
  '@lcc/tokens/css/motion.css',
  '@lcc/tokens/css/tier.css',
] as const;

// TypeScript
export * from './ts/colors';
export * from './ts/spacing';
export * from './ts/typography';
export * from './ts/radius';
export * from './ts/shadow';
export * from './ts/motion';
export * from './ts/tier';
