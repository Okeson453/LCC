/**
 * Currency formatting helper.
 */

const DEFAULT_LOCALE = 'en-US';
const DEFAULT_CURRENCY = 'USD';

export interface FormatCurrencyOptions {
  locale?: string;
  currency?: string;
  maximumFractionDigits?: number;
}

export function formatCurrency(
  value: number,
  opts: FormatCurrencyOptions = {},
): string {
  return new Intl.NumberFormat(opts.locale ?? DEFAULT_LOCALE, {
    style: 'currency',
    currency: opts.currency ?? DEFAULT_CURRENCY,
    maximumFractionDigits: opts.maximumFractionDigits,
  }).format(value);
}
