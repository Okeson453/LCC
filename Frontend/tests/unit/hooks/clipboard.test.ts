import { describe, expect, it, vi } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useClipboard } from '@/hooks/use-clipboard';

describe('useClipboard', () => {
  it('writes text', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.assign(navigator, { clipboard: { writeText } });
    const { result } = renderHook(() => useClipboard());
    await act(async () => {
      await result.current.copy('hello');
    });
    expect(writeText).toHaveBeenCalledWith('hello');
    expect(result.current.copied).toBe(true);
  });
});
