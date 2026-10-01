/**
 * View-model for the composer.
 */

import type { ContentVariantKind, KbCitation } from '@lcc/api-types';

export interface ComposerVariantViewModel {
  variant: ContentVariantKind;
  body: string;
  kbRefs: KbCitation[];
  groundingScore: number;
}

export interface ComposerState {
  prompt: string;
  selectedVariant: ContentVariantKind | null;
  body: string;
  isComposing: boolean;
  variants: ComposerVariantViewModel[];
}
