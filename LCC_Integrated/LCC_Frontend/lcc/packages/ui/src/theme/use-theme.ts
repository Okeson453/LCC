import { useTheme as useNextTheme } from 'next-themes';
import type { ThemeMode } from './ThemeProvider';

export function useTheme(): {
  theme: ThemeMode | undefined;
  resolvedTheme: 'light' | 'dark' | 'high_contrast' | undefined;
  setTheme: (mode: ThemeMode) => void;
} {
  const { theme, resolvedTheme, setTheme } = useNextTheme();
  return {
    theme: theme as ThemeMode | undefined,
    resolvedTheme: resolvedTheme as 'light' | 'dark' | 'high_contrast' | undefined,
    setTheme: (mode) => setTheme(mode),
  };
}
