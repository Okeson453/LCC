/**
 * Locale loader — reads JSON files synchronously and exports them as a
 * locale → namespace → messages map.
 */

import commonEn from './locales/en/common.json';
import approvalEn from './locales/en/approval.json';
import contentEn from './locales/en/content.json';
import engagementEn from './locales/en/engagement.json';
import outreachEn from './locales/en/outreach.json';
import opportunityEn from './locales/en/opportunity.json';
import analyticsEn from './locales/en/analytics.json';
import copilotEn from './locales/en/copilot.json';
import errorsEn from './locales/en/errors.json';

import commonEs from './locales/es/common.json';
import approvalEs from './locales/es/approval.json';
import contentEs from './locales/es/content.json';
import engagementEs from './locales/es/engagement.json';
import outreachEs from './locales/es/outreach.json';
import opportunityEs from './locales/es/opportunity.json';
import analyticsEs from './locales/es/analytics.json';
import copilotEs from './locales/es/copilot.json';
import errorsEs from './locales/es/errors.json';

import commonDe from './locales/de/common.json';
import approvalDe from './locales/de/approval.json';
import contentDe from './locales/de/content.json';
import engagementDe from './locales/de/engagement.json';
import outreachDe from './locales/de/outreach.json';
import opportunityDe from './locales/de/opportunity.json';
import analyticsDe from './locales/de/analytics.json';
import copilotDe from './locales/de/copilot.json';
import errorsDe from './locales/de/errors.json';

import commonFr from './locales/fr/common.json';
import approvalFr from './locales/fr/approval.json';
import contentFr from './locales/fr/content.json';
import engagementFr from './locales/fr/engagement.json';
import outreachFr from './locales/fr/outreach.json';
import opportunityFr from './locales/fr/opportunity.json';
import analyticsFr from './locales/fr/analytics.json';
import copilotFr from './locales/fr/copilot.json';
import errorsFr from './locales/fr/errors.json';

import type { Locale } from './config';

const messagesByLocale: Record<Locale, Record<string, Record<string, unknown>>> = {
  en: {
    common: commonEn,
    approval: approvalEn,
    content: contentEn,
    engagement: engagementEn,
    outreach: outreachEn,
    opportunity: opportunityEn,
    analytics: analyticsEn,
    copilot: copilotEn,
    errors: errorsEn,
  },
  es: {
    common: commonEs,
    approval: approvalEs,
    content: contentEs,
    engagement: engagementEs,
    outreach: outreachEs,
    opportunity: opportunityEs,
    analytics: analyticsEs,
    copilot: copilotEs,
    errors: errorsEs,
  },
  de: {
    common: commonDe,
    approval: approvalDe,
    content: contentDe,
    engagement: engagementDe,
    outreach: outreachDe,
    opportunity: opportunityDe,
    analytics: analyticsDe,
    copilot: copilotDe,
    errors: errorsDe,
  },
  fr: {
    common: commonFr,
    approval: approvalFr,
    content: contentFr,
    engagement: engagementFr,
    outreach: outreachFr,
    opportunity: opportunityFr,
    analytics: analyticsFr,
    copilot: copilotFr,
    errors: errorsFr,
  },
};

export function getMessages(locale: Locale): Record<string, Record<string, unknown>> {
  return messagesByLocale[locale];
}

export function getAllLocales(): Locale[] {
  return Object.keys(messagesByLocale) as Locale[];
}
