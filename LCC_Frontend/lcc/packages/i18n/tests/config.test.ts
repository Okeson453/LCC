import { describe, expect, it } from 'vitest';
import {
  DEFAULT_LOCALE,
  LOCALES,
  isSupportedLocale,
  resolveLocale,
} from '../src/config';

describe('i18n config', () => {
  it('exports en as the default locale', () => {
    expect(DEFAULT_LOCALE).toBe('en');
  });

  it('includes the launch set of locales', () => {
    expect(LOCALES).toContain('en');
  });

  it('isSupportedLocale correctly identifies supported and unsupported', () => {
    expect(isSupportedLocale('en')).toBe(true);
    expect(isSupportedLocale('zh')).toBe(false);
  });

  it('resolveLocale parses Accept-Language headers', () => {
    expect(resolveLocale('en-US,en;q=0.9')).toBe('en');
    expect(resolveLocale('es-ES,es;q=0.9')).toBe('es');
    expect(resolveLocale('de-DE')).toBe('de');
    expect(resolveLocale('ja-JP')).toBe('en'); // fallback
    expect(resolveLocale(null)).toBe('en');
  });
});
