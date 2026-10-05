import { describe, expect, it, vi } from 'vitest';
import { groupCitations, assertCitationsPresent } from '../src/utils/citation-grouper';
import type { KbCitation } from '@lcc/api-types';

const sample: KbCitation[] = [
  { recordId: 'r1', title: 'Senior PM resume', category: 'resume', excerpt: '...', url: null },
  { recordId: 'r2', title: 'Fintech case study', category: 'case_study', excerpt: '...', url: null },
  { recordId: 'r3', title: 'ICP', category: 'ideal_customer', excerpt: '...', url: null },
  { recordId: 'r4', title: 'About me', category: 'resume', excerpt: '...', url: null },
];

describe('groupCitations', () => {
  it('groups by category preserving canonical order', () => {
    const groups = groupCitations(sample);
    expect(groups.map((g) => g.category)).toEqual(['ideal_customer', 'resume', 'case_study']);
  });

  it('groups multiple citations in the same category together', () => {
    const groups = groupCitations(sample);
    const resume = groups.find((g) => g.category === 'resume');
    expect(resume?.citations.length).toBe(2);
  });

  it('returns empty array for empty input', () => {
    expect(groupCitations([])).toEqual([]);
  });
});

describe('assertCitationsPresent', () => {
  it('throws in development when citations are empty', () => {
    // `Object.defineProperty(process.env, …)` is rejected by Node
    // ("only accepts a configurable, writable, and enumerable data
    // descriptor"); `vi.stubEnv` is the supported way to override it.
    vi.stubEnv('NODE_ENV', 'development');
    try {
      expect(() => assertCitationsPresent([], 'test')).toThrow(/missing KB citations/);
    } finally {
      vi.unstubAllEnvs();
    }
  });

  it('does not throw in production but warns', () => {
    vi.stubEnv('NODE_ENV', 'production');
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
    try {
      expect(() => assertCitationsPresent([], 'test')).not.toThrow();
      expect(warnSpy).toHaveBeenCalled();
    } finally {
      warnSpy.mockRestore();
      vi.unstubAllEnvs();
    }
  });

  it('does not throw when citations present', () => {
    expect(() => assertCitationsPresent(sample, 'test')).not.toThrow();
  });
});
