/**
 * Root layout — `<html>`, providers, fonts, theme.
 */

import type { Metadata, Viewport } from 'next';
import { Inter } from 'next/font/google';
import { ThemeProvider } from '@lcc/ui';
import { I18nProvider, getMessages, DEFAULT_LOCALE } from '@lcc/i18n';
import { SkipNav } from '@lcc/ui';
import { Providers } from './providers';
import '../styles/globals.css';

const inter = Inter({ subsets: ['latin'], display: 'swap', variable: '--font-inter' });

export const metadata: Metadata = {
  title: {
    default: 'LinkedIn Manager',
    template: '%s · LinkedIn Manager',
  },
  description: 'Compliance-aware LinkedIn copilot',
  manifest: '/manifest.json',
  applicationName: 'LinkedIn Manager',
  appleWebApp: {
    capable: true,
    title: 'LinkedIn Manager',
    statusBarStyle: 'black-translucent',
  },
  formatDetection: { telephone: false },
};

export const viewport: Viewport = {
  themeColor: '#0c1019',
  width: 'device-width',
  initialScale: 1,
  maximumScale: 1,
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}): React.ReactElement {
  return (
    <html lang="en" suppressHydrationWarning>
      <body className={inter.variable}>
        <script
          dangerouslySetInnerHTML={{ __html: `
            if ('serviceWorker' in navigator) {
              window.addEventListener('load', () => {
                navigator.serviceWorker.register('/sw.js').catch(() => undefined);
              });
            }
          ` }}
        />
        <ThemeProvider defaultTheme="dark" storageKey="lcc.theme">
          <I18nProvider locale={DEFAULT_LOCALE} messages={getMessages(DEFAULT_LOCALE)}>
            <SkipNav />
            <Providers>{children}</Providers>
          </I18nProvider>
        </ThemeProvider>
      </body>
    </html>
  );
}
