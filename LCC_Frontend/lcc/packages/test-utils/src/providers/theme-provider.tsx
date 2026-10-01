/**
 * ThemeProvider — next-themes test wrapper.
 */

import * as React from 'react';
import { ThemeProvider as NextThemesProvider } from 'next-themes';

export interface ThemeProviderProps {
  children: React.ReactNode;
  defaultTheme?: 'light' | 'dark' | 'high_contrast';
}

export function ThemeProvider({
  children,
  defaultTheme = 'dark',
}: ThemeProviderProps): React.ReactElement {
  return (
    <NextThemesProvider
      attribute="data-theme"
      defaultTheme={defaultTheme}
      enableSystem={false}
      disableTransitionOnChange
    >
      {children}
    </NextThemesProvider>
  );
}
