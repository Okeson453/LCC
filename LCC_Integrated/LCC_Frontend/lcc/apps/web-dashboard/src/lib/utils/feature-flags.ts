/**
 * Feature flags — client-side. Server-of-record is the backend's config;
 * this module just exposes typed accessors.
 */

const flags = {
  COPILOT_PROPOSALS: true,
  TRACK_B_EXTENSION: true,
  REALTIME_BRIEFING: true,
  REALTIME_APPROVALS: true,
  REALTIME_ENGAGEMENT: true,
  REALTIME_COMPLIANCE: true,
  REALTIME_SEQUENCE: true,
  VISUAL_REGRESSION: process.env.NODE_ENV !== 'production',
  PWA_ENABLED: true,
  CONTENT_VARIANTS_4: true,
  ANALYTICS_DIGEST_WEEKLY: true,
  ANALYTICS_DIGEST_MONTHLY: true,
  ONBOARDING_V2: false,
  COMPLIANCE_TWO_REVIEWER: true,
} as const;

export type FeatureFlag = keyof typeof flags;

export function isEnabled(flag: FeatureFlag): boolean {
  return flags[flag] === true;
}

export function getFlag<T extends FeatureFlag>(flag: T): boolean {
  return flags[flag];
}
