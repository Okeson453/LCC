import { describe, expect, it } from 'vitest';
import { cn } from '@lcc/ui';

describe('ui.cn', () => {
  it('combines strings', () => {
    expect(cn('a', 'b')).toBe('a b');
  });
  it('filters falsy', () => {
    expect(cn('a', false, null, 'b')).toBe('a b');
  });
});
