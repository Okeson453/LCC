import { describe, expect, it } from 'vitest';
import { groupCitations } from '@lcc/approval-gate';
import type { KbCitation } from '@lcc/api-types';

/**
 * These assertions target the API `@lcc/approval-gate` actually exports.
 *
 * The previous version imported `groupCitationsByKind`, a function the package
 * never had, and passed wire-shaped fixtures (`{record_id, kind, snippet}`).
 * The real helper is `groupCitations(citations)`, which groups presentation
 * `KbCitation`s by their `category` in a fixed canonical order and returns
 * `CitationGroup[]`.
 */
const sample: KbCitation[] = [
  { recordId: 'k1', title: 'A', category: 'resume', excerpt: '...', url: null },
  { recordId: 'k2', title: 'B', category: 'resume', excerpt: '...', url: null },
  { recordId: 'k3', title: 'C', category: 'voice_sample', excerpt: '...', url: null },
];

describe('citation grouper', () => {
  it('groups by category', () => {
    const groups = groupCitations(sample);
    expect(groups.map((g) => g.category).sort()).toEqual(['resume', 'voice_sample']);
  });

  it('keeps every citation in its category group', () => {
    const groups = groupCitations(sample);
    const resume = groups.find((g) => g.category === 'resume');
    const voice = groups.find((g) => g.category === 'voice_sample');
    expect(resume?.citations.length).toBe(2);
    expect(voice?.citations.length).toBe(1);
  });

  it('labels every group', () => {
    for (const g of groupCitations(sample)) {
      expect(g.label).toEqual(expect.any(String));
    }
  });
});
