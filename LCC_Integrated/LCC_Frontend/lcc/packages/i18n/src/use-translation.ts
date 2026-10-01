/**
 * Typed useTranslations wrapper. Provides autocomplete on namespace keys.
 */

import { useTranslations as useNextIntlTranslations } from 'next-intl';
import type { Namespace } from './config';

export type TypedTranslations<N extends Namespace> = {
  (key: string, params?: Record<string, string | number>): string;
  /** Strongly-typed accessor when key is a known string literal. */
  rich: (key: string, values: Record<string, (chunks: React.ReactNode) => React.ReactNode>) => React.ReactNode;
  /** Mark a string for translation without rendering. */
  mark: (key: string) => string;
};

export function useTranslation<N extends Namespace>(
  namespace: N,
): TypedTranslations<N> {
  const t = useNextIntlTranslations(namespace as never);
  return t as unknown as TypedTranslations<N>;
}
