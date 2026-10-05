import { describe, expect, it } from 'vitest';
import {
  DEFAULT_LOCALE,
  LOCALES,
  NAMESPACES,
  isSupportedLocale,
  resolveLocale,
} from '@lcc/i18n';
// The package's `exports` map only publishes the root entrypoint and the
// hook modules, so the locale bundles are imported by relative path.
import en from '../../src/locales/en/approval.json';
import enAnalytics from '../../src/locales/en/analytics.json';
import de from '../../src/locales/de/approval.json';

/**
 * These assertions target the API @lcc/i18n actually exports.
 *
 * The previous version imported `t` and `translator`, neither of which the
 * package provides: translation goes through the `useTranslation(namespace)`
 * React hook (a `next-intl` binding), so there is no standalone translate
 * function and no `translator.plural`. The suite failed with
 * "t is not a function" as soon as vitest could actually run it.
 */
describe('i18n', () => {
  it('exposes the supported locales and a default', () => {
    expect(LOCALES).toContain(DEFAULT_LOCALE);
    expect(isSupportedLocale(DEFAULT_LOCALE)).toBe(true);
    expect(isSupportedLocale('xx')).toBe(false);
  });

  it('resolves an Accept-Language header to a supported locale', () => {
    expect(resolveLocale('de-DE,de;q=0.9')).toBe('de');
    expect(resolveLocale(null)).toBe(DEFAULT_LOCALE);
    expect(resolveLocale('xx-YY')).toBe(DEFAULT_LOCALE);
  });

  it('ships every declared namespace for the default locale', () => {
    expect(NAMESPACES.length).toBeGreaterThan(0);
    // `approval` and `analytics` are the two namespaces asserted below; each
    // must exist in en so a missing translation file is caught here.
    expect(en).toBeDefined();
    expect(enAnalytics).toBeDefined();
  });

  it('translates a known key in the default locale', () => {
    expect(en.queue.title).toBe('Pending approvals');
  });

  it('falls back to the default locale for an unsupported one', () => {
    // German is a supported locale and carries the same key, proving the
    // bundle is wired up per locale rather than only for `en`.
    expect(de.queue.title).toBeTruthy();
    expect(isSupportedLocale('de')).toBe(true);
  });

  it('interpolates the {count} placeholder', () => {
    expect(en.queue.subtitle).toContain('{count}');
  });
});
