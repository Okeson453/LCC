/**
 * cn — class-name composition helper.
 * Combines clsx + tailwind-merge so duplicate Tailwind classes collapse.
 */

import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}
