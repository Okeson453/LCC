/**
 * TypeScript mirrors of the CSS color tokens.
 * Provides typed access to the palette for JS-only contexts (charts, canvas, etc.).
 */

export type Theme = 'dark' | 'light' | 'high-contrast';

export interface ColorPalette {
  background: string;
  foreground: string;
  card: string;
  cardForeground: string;
  popover: string;
  popoverForeground: string;
  border: string;
  input: string;
  ring: string;
  primary: string;
  primaryForeground: string;
  secondary: string;
  secondaryForeground: string;
  accent: string;
  accentForeground: string;
  muted: string;
  mutedForeground: string;
  destructive: string;
  destructiveForeground: string;
  success: string;
  successForeground: string;
  warning: string;
  warningForeground: string;
  info: string;
  infoForeground: string;
  tier1: string;
  tier2: string;
  tier3: string;
  tier4: string;
  tier5: string;
  restricted: string;
  restrictedBg: string;
}

export const colorsDark: ColorPalette = {
  background: 'hsl(222 47% 5%)',
  foreground: 'hsl(210 40% 98%)',
  card: 'hsl(222 47% 7%)',
  cardForeground: 'hsl(210 40% 98%)',
  popover: 'hsl(222 47% 9%)',
  popoverForeground: 'hsl(210 40% 98%)',
  border: 'hsl(217 33% 17%)',
  input: 'hsl(217 33% 17%)',
  ring: 'hsl(212 96% 60%)',
  primary: 'hsl(212 96% 60%)',
  primaryForeground: 'hsl(222 47% 5%)',
  secondary: 'hsl(217 33% 17%)',
  secondaryForeground: 'hsl(210 40% 98%)',
  accent: 'hsl(217 33% 17%)',
  accentForeground: 'hsl(210 40% 98%)',
  muted: 'hsl(217 33% 17%)',
  mutedForeground: 'hsl(215 20% 65%)',
  destructive: 'hsl(0 72% 51%)',
  destructiveForeground: 'hsl(210 40% 98%)',
  success: 'hsl(142 71% 45%)',
  successForeground: 'hsl(210 40% 98%)',
  warning: 'hsl(38 92% 50%)',
  warningForeground: 'hsl(222 47% 5%)',
  info: 'hsl(199 89% 48%)',
  infoForeground: 'hsl(210 40% 98%)',
  tier1: 'hsl(215 16% 47%)',
  tier2: 'hsl(212 96% 60%)',
  tier3: 'hsl(38 92% 50%)',
  tier4: 'hsl(20 90% 53%)',
  tier5: 'hsl(0 72% 51%)',
  restricted: 'hsl(0 84% 60%)',
  restrictedBg: 'hsl(0 84% 60% / 0.08)',
};

export const colorsLight: ColorPalette = {
  background: 'hsl(0 0% 100%)',
  foreground: 'hsl(222 47% 11%)',
  card: 'hsl(0 0% 100%)',
  cardForeground: 'hsl(222 47% 11%)',
  popover: 'hsl(0 0% 100%)',
  popoverForeground: 'hsl(222 47% 11%)',
  border: 'hsl(214 32% 91%)',
  input: 'hsl(214 32% 91%)',
  ring: 'hsl(212 96% 48%)',
  primary: 'hsl(212 96% 48%)',
  primaryForeground: 'hsl(0 0% 100%)',
  secondary: 'hsl(210 40% 96%)',
  secondaryForeground: 'hsl(222 47% 11%)',
  accent: 'hsl(210 40% 96%)',
  accentForeground: 'hsl(222 47% 11%)',
  muted: 'hsl(210 40% 96%)',
  mutedForeground: 'hsl(215 16% 47%)',
  destructive: 'hsl(0 72% 51%)',
  destructiveForeground: 'hsl(210 40% 98%)',
  success: 'hsl(142 71% 35%)',
  successForeground: 'hsl(0 0% 100%)',
  warning: 'hsl(32 95% 44%)',
  warningForeground: 'hsl(0 0% 100%)',
  info: 'hsl(199 89% 38%)',
  infoForeground: 'hsl(0 0% 100%)',
  tier1: 'hsl(215 16% 47%)',
  tier2: 'hsl(212 96% 48%)',
  tier3: 'hsl(32 95% 44%)',
  tier4: 'hsl(20 90% 44%)',
  tier5: 'hsl(0 72% 45%)',
  restricted: 'hsl(0 84% 50%)',
  restrictedBg: 'hsl(0 84% 50% / 0.08)',
};
