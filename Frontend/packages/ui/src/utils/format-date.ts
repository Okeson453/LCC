/**
 * Date formatting helpers — using `date-fns` under the hood.
 */

import {
  format,
  formatDistanceToNow,
  parseISO,
  isValid,
} from 'date-fns';

export type DateInput = string | number | Date;

export function formatDate(input: DateInput, pattern = 'PPP'): string {
  const d = typeof input === 'string' ? parseISO(input) : new Date(input);
  if (!isValid(d)) return '';
  return format(d, pattern);
}

export function formatTime(input: DateInput, pattern = 'p'): string {
  const d = typeof input === 'string' ? parseISO(input) : new Date(input);
  if (!isValid(d)) return '';
  return format(d, pattern);
}

export function formatDateTime(input: DateInput, pattern = 'PPP p'): string {
  const d = typeof input === 'string' ? parseISO(input) : new Date(input);
  if (!isValid(d)) return '';
  return format(d, pattern);
}

export function formatRelative(input: DateInput): string {
  const d = typeof input === 'string' ? parseISO(input) : new Date(input);
  if (!isValid(d)) return '';
  return formatDistanceToNow(d, { addSuffix: true });
}

export function isValidDate(input: DateInput): boolean {
  const d = typeof input === 'string' ? parseISO(input) : new Date(input);
  return isValid(d);
}
