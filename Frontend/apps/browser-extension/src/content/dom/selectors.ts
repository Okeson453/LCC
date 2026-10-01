/**
 * LinkedIn DOM selectors — kept narrow. Tier 1 read-only actions only.
 * Audit ref: M-23 + A-17.
 */

export const LINKEDIN_SELECTORS = {
  profileName: '[data-test-id="profile-name"], h1.text-heading-xlarge',
  profileHeadline: '[data-test-id="profile-headline"], div.text-body-medium.break-words',
  feedPosts: 'div.feed-shared-update-v2, div.occludable-update',
  inboxThread: 'div.msg-thread',
  companyName: 'h1.org-top-card-summary__title',
  actionMenu: 'button[aria-label="More actions"], button[aria-label*="More"]',
} as const;

export function safeQuery<T extends Element = Element>(root: ParentNode, sel: string): T | null {
  try { return root.querySelector<T>(sel); } catch { return null; }
}

export function safeQueries<T extends Element = Element>(root: ParentNode, sel: string): T[] {
  try { return Array.from(root.querySelectorAll<T>(sel)); } catch { return []; }
}
