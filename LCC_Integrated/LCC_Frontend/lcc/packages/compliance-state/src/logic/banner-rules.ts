/**
 * Banner copy rules — keys to localized strings for the restricted banner.
 * Per audit: keep i18n-friendly; this is the canonical key surface.
 */

export type BannerReason = 'cooldown' | 'pause' | 'gov_throttle' | 'gov_block' | null;

export interface BannerCopy {
  title: string;
  body: string;
  cta?: { label: string; href: string };
}

export function bannerCopyFor(reason: BannerReason): BannerCopy {
  switch (reason) {
    case 'cooldown':
      return {
        title: 'Account in cooldown',
        body: 'New actions are paused temporarily while we balance the queue.',
        cta: { label: 'Learn more', href: '/settings/notifications' },
      };
    case 'pause':
      return {
        title: 'Account paused',
        body: 'No actions until the pause is lifted. Existing scheduled items continue.',
      };
    case 'gov_throttle':
      return {
        title: 'Throttled',
        body: 'Reduced tier allowances during a governor-supervised window.',
      };
    case 'gov_block':
      return {
        title: 'Blocked by governor',
        body: 'A compliance rule paused this account. Contact support to resolve.',
        cta: { label: 'Contact support', href: '/restricted' },
      };
    default:
      return {
        title: 'Restricted state',
        body: 'Account restrictions are in effect.',
      };
  }
}
