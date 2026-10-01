/**
 * Compliance state derivation logic. Per audit reference: derives the
 * banner tone from the raw `{ restricted, reason, until }` shape.
 */

export type BannerTone = 'info' | 'warning' | 'danger';

export interface RestrictionInput {
  restricted: boolean;
  reason: string | null;
  until: string | null;
}

export function deriveBannerTone(input: RestrictionInput): BannerTone {
  if (!input.restricted) return 'info';
  if (input.reason === 'pause' || input.reason === 'gov_block') return 'danger';
  if (input.reason === 'cooldown' || input.reason === 'gov_throttle') return 'warning';
  return 'warning';
}

export function isExpired(input: RestrictionInput): boolean {
  if (!input.until) return false;
  return new Date(input.until).getTime() <= Date.now();
}
