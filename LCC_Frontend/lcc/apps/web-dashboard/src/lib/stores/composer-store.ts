/**
 * Composer store — in-progress draft state.
 *
 * Note: This is UI-only state for the active draft. The persisted draft
 * lives on the backend; the store just keeps the form values while the
 * user is composing.
 */

import { create } from 'zustand';
import type { ContentVariantKind } from '@lcc/api-types';

interface ComposerState {
  prompt: string;
  selectedVariant: ContentVariantKind | null;
  body: string;
  setPrompt: (prompt: string) => void;
  setSelectedVariant: (variant: ContentVariantKind | null) => void;
  setBody: (body: string) => void;
  reset: () => void;
}

export const useComposerStore = create<ComposerState>((set) => ({
  prompt: '',
  selectedVariant: null,
  body: '',
  setPrompt: (prompt) => set({ prompt }),
  setSelectedVariant: (selectedVariant) => set({ selectedVariant }),
  setBody: (body) => set({ body }),
  reset: () => set({ prompt: '', selectedVariant: null, body: '' }),
}));
