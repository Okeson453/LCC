/**
 * i18n configuration — locale list, default, fallback.
 */

export const LOCALES = ['en', 'es', 'de', 'fr'] as const;
export type Locale = (typeof LOCALES)[number];

export const DEFAULT_LOCALE: Locale = 'en';

export const LOCALE_LABELS: Record<Locale, string> = {
  en: 'English',
  es: 'Español',
  de: 'Deutsch',
  fr: 'Français',
};

export const RTL_LOCALES: readonly Locale[] = [];

export function isSupportedLocale(value: string): value is Locale {
  return (LOCALES as readonly string[]).includes(value);
}

export function resolveLocale(acceptLanguage: string | null): Locale {
  if (!acceptLanguage) return DEFAULT_LOCALE;
  const candidates = acceptLanguage
    .split(',')
    .map((c) => c.split(';')[0]?.trim().toLowerCase())
    .filter((c): c is string => Boolean(c));
  for (const c of candidates) {
    const primary = c.split('-')[0];
    if (primary && isSupportedLocale(primary)) return primary;
  }
  return DEFAULT_LOCALE;
}

export const NAMESPACES = [
  'common',
  'approval',
  'content',
  'engagement',
  'outreach',
  'opportunity',
  'analytics',
  'copilot',
  'errors',
] as const;

export type Namespace = (typeof NAMESPACES)[number];
