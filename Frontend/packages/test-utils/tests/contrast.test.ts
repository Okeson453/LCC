import { describe, expect, it } from 'vitest';
import { contrastRatio, parseRgb, meetsAA, meetsAAA } from '../src/a11y/contrast';

describe('contrast', () => {
  it('computes ratio for known pair', () => {
    // black on white: 21:1
    const r = contrastRatio({ r: 0, g: 0, b: 0 }, { r: 255, g: 255, b: 255 });
    expect(r).toBeCloseTo(21, 0);
  });

  it('parses rgb() strings', () => {
    expect(parseRgb('rgb(10, 20, 30)')).toEqual({ r: 10, g: 20, b: 30 });
    expect(parseRgb('rgba(10, 20, 30, 0.5)')).toEqual({ r: 10, g: 20, b: 30 });
    expect(parseRgb('not-rgb')).toBeNull();
  });

  it('meetsAA returns true for black on white', () => {
    expect(meetsAA('rgb(0,0,0)', 'rgb(255,255,255)')).toBe(true);
  });

  it('meetsAA returns false for low-contrast', () => {
    expect(meetsAA('rgb(120,120,120)', 'rgb(125,125,125)')).toBe(false);
  });

  it('meetsAAA requires a higher ratio', () => {
    // 4.6:1 — passes AA but not AAA for normal text
    const ratio = contrastRatio({ r: 120, g: 120, b: 120 }, { r: 0, g: 0, b: 0 });
    expect(meetsAA('rgb(120,120,120)', 'rgb(0,0,0)')).toBe(true);
    expect(meetsAAA('rgb(120,120,120)', 'rgb(0,0,0)')).toBe(ratio >= 7);
  });
});
