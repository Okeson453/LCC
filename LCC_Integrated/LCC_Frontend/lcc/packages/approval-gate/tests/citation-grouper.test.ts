import { describe, expect, it } from 'vitest';
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
    const original = process.env.NODE_ENV;
    Object.defineProperty(process.env, 'NODE_ENV', { value: 'development', writable: true, configurable: true });
    try {
      expect(() => assertCitationsPresent([], 'test')).toThrow(/missing KB citations/);
    } finally {
      Object.defineProperty(process.env, 'NODE_ENV', { value: original, writable: true, configurable: true });
    }
  });

  it('does not throw in production but warns', () => {
    const original = process.env.NODE_ENV;
    Object.defineProperty(process.env, 'NODE_ENV', { value: 'production', writable: true, configurable: true });
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
    try {
      expect(() => assertCitationsPresent([], 'test')).not.toThrow();
      expect(warnSpy).toHaveBeenCalled();
    } finally {
      warnSpy.mockRestore();
      Object.defineProperty(process.env, 'NODE_ENV', { value: original, writable: true, configurable: true });
    }
  });

  it('does not throw when citations present', () => {
    expect(() => assertCitationsPresent(sample, 'test')).not.toThrow();
  });
});

// Vitest's `vi` is auto-imported from the globals config.
declare const vi: { spyOn: <T, K extends keyof T>(obj: T, method: K) => { mockImplementation: (fn: (...args: unknown[]) => unknown) => { mockRestore: () => void } } };
