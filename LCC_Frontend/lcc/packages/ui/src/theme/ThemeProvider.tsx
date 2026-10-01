import * as React from 'react';
import { ThemeProvider as NextThemesProvider } from 'next-themes';

export type ThemeMode = 'light' | 'dark' | 'high_contrast' | 'system';

export interface ThemeProviderProps {
  children: React.ReactNode;
  defaultTheme?: ThemeMode;
  enableSystem?: boolean;
  /** Storage key for the user's choice. */
  storageKey?: string;
}

export function ThemeProvider({
  children,
  defaultTheme = 'dark',
  enableSystem = false,
  storageKey = 'lcc.theme',
}: ThemeProviderProps): React.ReactElement {
  return (
    <NextThemesProvider
      attribute="data-theme"
      defaultTheme={defaultTheme}
      enableSystem={enableSystem}
      storageKey={storageKey}
      disableTransitionOnChange
    >
      {children}
    </NextThemesProvider>
  );
}
