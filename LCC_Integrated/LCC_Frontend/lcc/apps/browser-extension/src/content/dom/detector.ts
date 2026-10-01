/**
 * DOM detection — identifies LinkedIn surfaces and surfaces them as typed
 * events. Audit ref: M-23 + A-17 (extension ↔ LinkedIn DOM).
 */

export type LinkedInSurface =
  | { kind: 'profile'; vanity: string }
  | { kind: 'company'; slug: string }
  | { kind: 'feed' }
  | { kind: 'inbox'; threadId: string | null };

const RX_PROFILE = /^\/in\/([^/?#]+)/i;
const RX_COMPANY = /^\/company\/([^/?#]+)/i;
const RX_INBOX = /^\/messaging\/thread\/([^/?#]+)/i;

export function detectSurface(url: string = location.href): LinkedInSurface | null {
  const path = (() => {
    try { return new URL(url).pathname; } catch { return ''; }
  })();
  if (!path) return null;
  const profile = RX_PROFILE.exec(path);
  if (profile?.[1]) return { kind: 'profile', vanity: decodeURIComponent(profile[1]) };
  const company = RX_COMPANY.exec(path);
  if (company?.[1]) return { kind: 'company', slug: decodeURIComponent(company[1]) };
  const inbox = RX_INBOX.exec(path);
  if (inbox) return { kind: 'inbox', threadId: inbox[1] ? decodeURIComponent(inbox[1]) : null };
  if (path === '/' || path.startsWith('/feed')) return { kind: 'feed' };
  return null;
}

export function isWhitelistedSurface(): boolean {
  return detectSurface() !== null;
}
