import { test, expect } from '@playwright/test';
import { parseLinkedInUrl } from '@/lib/linkedin';

test.describe('url parser', () => {
  test('profile', () => {
    const p = parseLinkedInUrl('https://www.linkedin.com/in/jane-doe/');
    expect(p.vanity).toBe('jane-doe');
  });
  test('company', () => {
    const p = parseLinkedInUrl('https://www.linkedin.com/company/acme');
    expect(p.slug).toBe('acme');
  });
  test('feed', () => {
    const p = parseLinkedInUrl('https://www.linkedin.com/feed/');
    expect(p.feed).toBe(true);
  });
  test('thread', () => {
    const p = parseLinkedInUrl('https://www.linkedin.com/messaging/thread/abc-123/');
    expect(p.threadId).toBe('abc-123');
  });
  test('other host', () => {
    const p = parseLinkedInUrl('https://example.com/');
    expect(p.vanity).toBe(null);
  });
});
