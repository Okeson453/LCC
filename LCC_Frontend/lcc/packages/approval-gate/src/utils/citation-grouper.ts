/**
 * Citation grouping for `<KbCitationsList>` rendering.
 */

import type { KbCitation, KbCategory } from '@lcc/api-types';

export interface CitationGroup {
  category: KbCategory;
  label: string;
  citations: KbCitation[];
}

const CATEGORY_LABEL: Record<KbCategory, string> = {
  resume: 'Resume',
  portfolio: 'Portfolio',
  voice_sample: 'Voice sample',
  case_study: 'Case study',
  ideal_customer: 'ICP',
  content_pillar: 'Content pillar',
  goal: 'Goal',
};

const CATEGORY_ORDER: KbCategory[] = [
  'goal',
  'ideal_customer',
  'voice_sample',
  'resume',
  'case_study',
  'portfolio',
  'content_pillar',
];

export function groupCitations(citations: readonly KbCitation[]): CitationGroup[] {
  const map = new Map<KbCategory, KbCitation[]>();
  for (const c of citations) {
    const arr = map.get(c.category) ?? [];
    arr.push(c);
    map.set(c.category, arr);
  }
  const out: CitationGroup[] = [];
  for (const cat of CATEGORY_ORDER) {
    const arr = map.get(cat);
    if (arr && arr.length > 0) {
      out.push({ category: cat, label: CATEGORY_LABEL[cat], citations: arr });
    }
  }
  // Append any categories not in the order list (defensive)
  for (const [cat, arr] of map) {
    if (!CATEGORY_ORDER.includes(cat) && arr.length > 0) {
      out.push({ category: cat, label: CATEGORY_LABEL[cat], citations: arr });
    }
  }
  return out;
}

/**
 * Validates that a citation list satisfies Non-Negotiable §5 (axiom 5):
 * "Every artifact shown for approval must display its KB grounding citations."
 *
 * Empty list is a violation — throws in dev, fails silently in production but
 * logs to telemetry.
 */
export function assertCitationsPresent(
  citations: readonly KbCitation[] | undefined,
  context: string,
): void {
  if (!citations || citations.length === 0) {
    if (process.env.NODE_ENV === 'development') {
      throw new Error(
        `[approval-gate] ${context}: missing KB citations. Every approval must show grounding.`,
      );
    }
    if (typeof console !== 'undefined') {
      console.warn(
        `[approval-gate] ${context}: empty KB citation list — audit-grade event.`,
      );
    }
  }
}
