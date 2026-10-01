/**
 * truncate — text truncation helpers.
 */

export function truncate(text: string, max = 100, ellipsis = '…'): string {
  if (text.length <= max) return text;
  return text.slice(0, max - ellipsis.length).trimEnd() + ellipsis;
}

export function truncateWords(text: string, maxWords = 30, ellipsis = '…'): string {
  const words = text.split(/\s+/);
  if (words.length <= maxWords) return text;
  return words.slice(0, maxWords).join(' ') + ellipsis;
}
