/**
 * KB citation adapters.
 *
 * The wire types (`KbCitationRef`, generated from the OpenAPI contract) are
 * snake_case and carry `category: string`. The presentation type `KbCitation`
 * that `ApprovalDialog` / `KbCitationsList` render is camelCase and narrows
 * `category` to the `KbCategory` union.
 *
 * Passing `kb_refs` straight into `kbRefs={…}` therefore failed to typecheck
 * in every approval, content, outreach and proposal surface. This module is the
 * single place that maps between the two.
 */

import type { KbCitation, KbCategory } from '@lcc/api-types';
import type { KbCitationRef } from '@lcc/api-types';

/** Categories the UI knows how to label; anything else falls back to `goal`. */
const KNOWN_CATEGORIES: readonly KbCategory[] = [
  'resume',
  'portfolio',
  'voice_sample',
  'case_study',
  'ideal_customer',
  'content_pillar',
  'goal',
];

/**
 * Narrow a wire `category` string to the `KbCategory` union.
 *
 * The contract and the generated types disagree on this enum (the canonical
 * `KbRecord.category` allows skill/experience/project/tech/…, the generated
 * `KbCitationRef` allows the presentation set), so the value is validated at
 * runtime rather than cast.
 */
export function toKbCategory(value: string): KbCategory {
  return (KNOWN_CATEGORIES as readonly string[]).includes(value)
    ? (value as KbCategory)
    : 'goal';
}

/** Convert one wire citation into the presentation shape. */
export function toKbCitation(ref: KbCitationRef): KbCitation {
  return {
    recordId: ref.record_id,
    title: ref.title,
    category: toKbCategory(ref.category),
    excerpt: ref.excerpt,
    url: ref.url,
  };
}

/** Convert a list of wire citations, dropping malformed entries. */
export function toKbCitations(refs: readonly KbCitationRef[] | null | undefined): KbCitation[] {
  if (!refs) return [];
  return refs.filter((r): r is KbCitationRef => Boolean(r?.record_id)).map(toKbCitation);
}
