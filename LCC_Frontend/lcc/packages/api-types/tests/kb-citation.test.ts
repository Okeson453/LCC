import { describe, expect, it } from 'vitest';
import {
  groupCitationsByCategory,
  isValidCitationList,
  KB_CATEGORY_LABELS,
  type KbCitation,
} from '../src/manual/kb-citation';

const sampleCitations: KbCitation[] = [
  {
    recordId: 'r1',
    title: 'Senior PM resume',
    category: 'resume',
    excerpt: '...',
    url: null,
  },
  {
    recordId: 'r2',
    title: 'Fintech case study',
    category: 'case_study',
    excerpt: '...',
    url: null,
  },
  {
    recordId: 'r3',
    title: 'ICP definition',
    category: 'ideal_customer',
    excerpt: '...',
    url: null,
  },
];

describe('kb-citation', () => {
  it('groups by category', () => {
    const grouped = groupCitationsByCategory(sampleCitations);
    expect(grouped.size).toBe(3);
    expect(grouped.get('resume')?.length).toBe(1);
    expect(grouped.get('case_study')?.length).toBe(1);
    expect(grouped.get('ideal_customer')?.length).toBe(1);
  });

  it('isValidCitationList rejects empty', () => {
    expect(isValidCitationList([])).toBe(false);
    expect(isValidCitationList(sampleCitations)).toBe(true);
  });

  it('KB_CATEGORY_LABELS covers all categories', () => {
    const categories: KbCitation['category'][] = [
      'resume',
      'portfolio',
      'voice_sample',
      'case_study',
      'ideal_customer',
      'content_pillar',
      'goal',
    ];
    for (const c of categories) {
      expect(KB_CATEGORY_LABELS[c]).toBeDefined();
    }
  });
});
