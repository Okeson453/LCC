/**
 * KB Citation — the grounding evidence shown in every ApprovalDialog.
 *
 * Per Non-Negotiable §5 (axiom 5): "every artifact shown for approval must
 * display its KB grounding citations." This type defines the shape the
 * frontend renders; the backend supplies these via `Approval.kb_refs`.
 */

export interface KbCitation {
  /** KB record ID (opaque). */
  recordId: string;
  title: string;
  category: KbCategory;
  /** Short excerpt from the KB record. */
  excerpt: string;
  /** Optional URL for "open in KB" link. */
  url: string | null;
}

export type KbCategory =
  | 'resume'
  | 'portfolio'
  | 'voice_sample'
  | 'case_study'
  | 'ideal_customer'
  | 'content_pillar'
  | 'goal';

export const KB_CATEGORY_LABELS: Record<KbCategory, string> = {
  resume: 'Resume',
  portfolio: 'Portfolio',
  voice_sample: 'Voice sample',
  case_study: 'Case study',
  ideal_customer: 'ICP',
  content_pillar: 'Content pillar',
  goal: 'Goal',
};

export function groupCitationsByCategory(
  citations: readonly KbCitation[],
): Map<KbCategory, KbCitation[]> {
  const grouped = new Map<KbCategory, KbCitation[]>();
  for (const c of citations) {
    const list = grouped.get(c.category) ?? [];
    list.push(c);
    grouped.set(c.category, list);
  }
  return grouped;
}

/**
 * Validates that a citation list is non-empty. Used by ApprovalDialog to
 * enforce Non-Negotiable §5 — a citationless approval is a UI violation.
 */
export function isValidCitationList(citations: readonly KbCitation[]): boolean {
  return citations.length > 0;
}
