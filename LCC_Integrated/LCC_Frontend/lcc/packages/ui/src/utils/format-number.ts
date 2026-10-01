/**
 * Number formatting helpers.
 */

const DEFAULT_LOCALE = 'en-US';

export interface FormatNumberOptions {
  locale?: string;
  minimumFractionDigits?: number;
  maximumFractionDigits?: number;
  notation?: 'standard' | 'compact' | 'scientific' | 'engineering';
}

export function formatNumber(value: number, opts: FormatNumberOptions = {}): string {
  return new Intl.NumberFormat(opts.locale ?? DEFAULT_LOCALE, {
    minimumFractionDigits: opts.minimumFractionDigits,
    maximumFractionDigits: opts.maximumFractionDigits,
    notation: opts.notation,
  }).format(value);
}

export function formatPercent(value: number, opts: FormatNumberOptions = {}): string {
  return new Intl.NumberFormat(opts.locale ?? DEFAULT_LOCALE, {
    style: 'percent',
    minimumFractionDigits: opts.minimumFractionDigits ?? 0,
    maximumFractionDigits: opts.maximumFractionDigits ?? 1,
  }).format(value);
}

export function formatInteger(value: number, opts: FormatNumberOptions = {}): string {
  return formatNumber(value, { ...opts, maximumFractionDigits: 0 });
}
