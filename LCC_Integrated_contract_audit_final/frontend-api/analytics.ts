/**
 * Analytics API — content, profile, network, outreach, funnels, digests.
 *
 * Canonical paths from `config.ts::API_PATHS.analytics`.
 */

import { apiFetch } from './client';
import { API_PATHS } from './config';
import type {
  AccountHealth,
  ContentAnalytics,
  Digest,
  FunnelAnalytics,
  NetworkAnalytics,
  OutreachAnalytics,
  ProfileAnalytics,
} from '@lcc/api-types';

export async function fetchContentAnalytics(_memberId: string): Promise<ContentAnalytics> {
  return apiFetch<ContentAnalytics>(`/api/v1/analytics/content`);
}

export async function fetchProfileAnalytics(_memberId: string): Promise<ProfileAnalytics> {
  return apiFetch<ProfileAnalytics>(`/api/v1/analytics/profile`);
}

export async function fetchNetworkAnalytics(_memberId: string): Promise<NetworkAnalytics> {
  return apiFetch<NetworkAnalytics>(`/api/v1/analytics/network`);
}

export async function fetchOutreachAnalytics(_memberId: string): Promise<OutreachAnalytics> {
  return apiFetch<OutreachAnalytics>(`/api/v1/analytics/outreach`);
}

export async function fetchJobFunnel(_memberId: string): Promise<FunnelAnalytics> {
  return apiFetch<FunnelAnalytics>(`/api/v1/analytics/funnel/job`);
}

export async function fetchClientFunnel(_memberId: string): Promise<FunnelAnalytics> {
  return apiFetch<FunnelAnalytics>(`/api/v1/analytics/funnel/client`);
}

export async function fetchAccountHealth(_memberId: string): Promise<AccountHealth> {
  return apiFetch<AccountHealth>(`/api/v1/analytics/account-health`);
}

export async function fetchWeeklyDigest(_memberId: string): Promise<Digest> {
  return apiFetch<Digest>(`/api/v1/analytics/digest/weekly`);
}

export async function fetchMonthlyDigest(_memberId: string): Promise<Digest> {
  return apiFetch<Digest>(`/api/v1/analytics/digest/monthly`);
}

export async function fetchDashboard(
  memberId: string,
  start: string,
  end: string,
): Promise<unknown> {
  return apiFetch(API_PATHS.analytics.dashboard, {
    query: { start, end, member_id: memberId },
  });
}

const ANALYTICS_DISPATCH: Record<string, (memberId: string) => Promise<unknown>> = {
  content: fetchContentAnalytics,
  profile: fetchProfileAnalytics,
  network: fetchNetworkAnalytics,
  outreach: fetchOutreachAnalytics,
  'account-health': fetchAccountHealth,
  'funnel/job': fetchJobFunnel,
  'funnel/client': fetchClientFunnel,
  'digest/weekly': fetchWeeklyDigest,
  'digest/monthly': fetchMonthlyDigest,
};

export async function getAnalytics(memberId: string, metric: string): Promise<unknown> {
  const loader = ANALYTICS_DISPATCH[metric];
  if (!loader) {
    throw new Error(
      `[getAnalytics] unknown metric "${metric}". Supported: ${Object.keys(ANALYTICS_DISPATCH).join(', ')}`,
    );
  }
  return loader(memberId);
}
