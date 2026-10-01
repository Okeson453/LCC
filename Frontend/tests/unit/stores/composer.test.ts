import { describe, expect, it } from 'vitest';
import { useComposerStore } from '@/lib/stores';

describe('composer store', () => {
  it('sets prompt and body', () => {
    useComposerStore.getState().setPrompt('hello world');
    useComposerStore.getState().setBody('draft body');
    expect(useComposerStore.getState().prompt).toBe('hello world');
    expect(useComposerStore.getState().body).toBe('draft body');
  });

  it('resets', () => {
    useComposerStore.getState().setPrompt('x');
    useComposerStore.getState().reset();
    expect(useComposerStore.getState().prompt).toBe('');
    expect(useComposerStore.getState().body).toBe('');
  });
});
