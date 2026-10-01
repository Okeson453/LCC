import { describe, expect, it } from 'vitest';
import { translator, t } from '@lcc/i18n';

describe('i18n', () => {
  it('translates known key in default locale', () => {
    expect(t('approvals.title', { locale: 'en' })).toBeTruthy();
  });

  it('falls back to default locale', () => {
    const out = t('approvals.title', { locale: 'de' as never });
    expect(out).toBeTruthy();
  });

  it('pluralizes', () => {
    expect(translator.plural('analytics.days', 1, undefined, 'en')).toContain('1');
    expect(translator.plural('analytics.days', 5, undefined, 'en')).toContain('5');
  });
});
