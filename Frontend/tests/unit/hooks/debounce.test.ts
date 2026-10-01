import { describe, expect, it, vi } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useDebounce } from '@/hooks/use-debounce';

describe('useDebounce', () => {
  it('delays updates', async () => {
    vi.useFakeTimers();
    const { result } = renderHook(() => useDebounce('hello', 100));
    expect(result.current).toBe('');
    act(() => vi.advanceTimersByTime(150));
    expect(result.current).toBe('hello');
    vi.useRealTimers();
  });
});
