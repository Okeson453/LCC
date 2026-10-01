/**
 * LinkedIn URL parser — extracts vanity, slug, urn, thread keys.
 * Audit ref: M-30.
 */

export interface LinkedInUrlParts {
  vanity: string | null;
  slug: string | null;
  urn: string | null;
  threadId: string | null;
  feed: boolean;
}

export function parseLinkedInUrl(raw: string): LinkedInUrlParts {
  const result: LinkedInUrlParts = { vanity: null, slug: null, urn: null, threadId: null, feed: false };
  let url: URL;
  try { url = new URL(raw); } catch { return result; }
  if (!url.hostname.endsWith('linkedin.com')) return result;

  const profile = /^\/in\/([^/?#]+)/i.exec(url.pathname);
  if (profile?.[1]) result.vanity = decodeURIComponent(profile[1]);

  const company = /^\/company\/([^/?#]+)/i.exec(url.pathname);
  if (company?.[1]) result.slug = decodeURIComponent(company[1]);

  const thread = /^\/messaging\/thread\/([^/?#]+)/i.exec(url.pathname);
  if (thread?.[1]) result.threadId = decodeURIComponent(thread[1]);

  const urn = /urn:li:([^:&]+):([^&]+)/.exec(raw);
  if (urn) result.urn = `urn:li:${urn[1]}:${urn[2]}`;

  if (url.pathname === '/' || url.pathname.startsWith('/feed')) result.feed = true;
  return result;
}
