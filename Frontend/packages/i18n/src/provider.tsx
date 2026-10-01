/**
 * I18nProvider — wraps the app and provides the locale + messages context.
 */

import * as React from 'react';
import { NextIntlClientProvider } from 'next-intl';
import { DEFAULT_LOCALE, type Locale } from './config';

export interface I18nProviderProps {
  children: React.ReactNode;
  locale?: Locale;
  /** Messages keyed by namespace. */
  messages?: Record<string, Record<string, unknown>>;
}

export function I18nProvider({
  children,
  locale = DEFAULT_LOCALE,
  messages = {},
}: I18nProviderProps): React.ReactElement {
  return (
    <NextIntlClientProvider
      locale={locale}
      messages={messages as never}
      timeZone="UTC"
      now={new Date()}
    >
      {children}
    </NextIntlClientProvider>
  );
}
