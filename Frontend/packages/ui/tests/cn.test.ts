import { describe, expect, it } from 'vitest';
import { cn } from '../src/utils/cn';

describe('cn', () => {
  it('joins class names', () => {
    expect(cn('a', 'b', 'c')).toBe('a b c');
  });

  it('drops falsy values', () => {
    expect(cn('a', undefined, null, false, '', 'b')).toBe('a b');
  });

  it('supports arrays', () => {
    expect(cn(['a', 'b'], 'c')).toBe('a b c');
  });

  it('supports objects (clsx style)', () => {
    expect(cn({ a: true, b: false, c: true })).toBe('a c');
  });

  it('deduplicates conflicting Tailwind classes', () => {
    expect(cn('p-2', 'p-4')).toBe('p-4');
    expect(cn('text-red-500', 'text-blue-500')).toBe('text-blue-500');
  });
});
