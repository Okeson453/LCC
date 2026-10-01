/**
 * colorContrast — minimal WCAG 2.x contrast ratio calculator.
 *
 * Use to verify that a foreground/background pair meets 4.5:1 (AA body text)
 * or 3:1 (AA large text or UI components).
 */

export function relativeLuminance(rgb: { r: number; g: number; b: number }): number {
  const linearize = (c: number) => {
    const v = c / 255;
    return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4);
  };
  return 0.2126 * linearize(rgb.r) + 0.7152 * linearize(rgb.g) + 0.0722 * linearize(rgb.b);
}

export function contrastRatio(
  fg: { r: number; g: number; b: number },
  bg: { r: number; g: number; b: number },
): number {
  const l1 = relativeLuminance(fg);
  const l2 = relativeLuminance(bg);
  const lighter = Math.max(l1, l2);
  const darker = Math.min(l1, l2);
  return (lighter + 0.05) / (darker + 0.05);
}

export function parseRgb(input: string): { r: number; g: number; b: number } | null {
  const match = input.match(/rgba?\(\s*(\d+)[,\s]+(\d+)[,\s]+(\d+)/);
  if (!match) return null;
  return { r: Number(match[1]), g: Number(match[2]), b: Number(match[3]) };
}

export function meetsAA(fg: string, bg: string, largeText = false): boolean {
  const f = parseRgb(fg);
  const b = parseRgb(bg);
  if (!f || !b) return false;
  const ratio = contrastRatio(f, b);
  return largeText ? ratio >= 3 : ratio >= 4.5;
}

export function meetsAAA(fg: string, bg: string, largeText = false): boolean {
  const f = parseRgb(fg);
  const b = parseRgb(bg);
  if (!f || !b) return false;
  const ratio = contrastRatio(f, b);
  return largeText ? ratio >= 4.5 : ratio >= 7;
}
