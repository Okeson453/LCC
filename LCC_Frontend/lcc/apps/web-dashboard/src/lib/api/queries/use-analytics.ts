import { useQuery, type UseQueryResult } from '@tanstack/react-query';
import {
  fetchContentAnalytics,
  fetchProfileAnalytics,
  fetchNetworkAnalytics,
  fetchOutreachAnalytics,
  fetchJobFunnel,
  fetchClientFunnel,
  fetchWeeklyDigest,
  fetchMonthlyDigest,
} from '../analytics';
import type {
  ContentAnalytics,
  Digest,
  FunnelAnalytics,
  NetworkAnalytics,
  OutreachAnalytics,
  ProfileAnalytics,
} from '@lcc/api-types';

export function useContentAnalytics(
  memberId: string | undefined,
): UseQueryResult<ContentAnalytics, Error> {
  return useQuery({
    queryKey: ['analytics', 'content', memberId] as const,
    queryFn: () => fetchContentAnalytics(memberId!),
    enabled: Boolean(memberId),
  });
}

export function useProfileAnalytics(
  memberId: string | undefined,
): UseQueryResult<ProfileAnalytics, Error> {
  return useQuery({
    queryKey: ['analytics', 'profile', memberId] as const,
    queryFn: () => fetchProfileAnalytics(memberId!),
    enabled: Boolean(memberId),
  });
}

export function useNetworkAnalytics(
  memberId: string | undefined,
): UseQueryResult<NetworkAnalytics, Error> {
  return useQuery({
    queryKey: ['analytics', 'network', memberId] as const,
    queryFn: () => fetchNetworkAnalytics(memberId!),
    enabled: Boolean(memberId),
  });
}

export function useOutreachAnalytics(
  memberId: string | undefined,
): UseQueryResult<OutreachAnalytics, Error> {
  return useQuery({
    queryKey: ['analytics', 'outreach', memberId] as const,
    queryFn: () => fetchOutreachAnalytics(memberId!),
    enabled: Boolean(memberId),
  });
}

export function useJobFunnel(memberId: string | undefined): UseQueryResult<FunnelAnalytics, Error> {
  return useQuery({
    queryKey: ['analytics', 'funnel', 'job', memberId] as const,
    queryFn: () => fetchJobFunnel(memberId!),
    enabled: Boolean(memberId),
  });
}

export function useClientFunnel(memberId: string | undefined): UseQueryResult<FunnelAnalytics, Error> {
  return useQuery({
    queryKey: ['analytics', 'funnel', 'client', memberId] as const,
    queryFn: () => fetchClientFunnel(memberId!),
    enabled: Boolean(memberId),
  });
}

export function useWeeklyDigest(memberId: string | undefined): UseQueryResult<Digest, Error> {
  return useQuery({
    queryKey: ['analytics', 'digest', 'weekly', memberId] as const,
    queryFn: () => fetchWeeklyDigest(memberId!),
    enabled: Boolean(memberId),
  });
}

export function useMonthlyDigest(memberId: string | undefined): UseQueryResult<Digest, Error> {
  return useQuery({
    queryKey: ['analytics', 'digest', 'monthly', memberId] as const,
    queryFn: () => fetchMonthlyDigest(memberId!),
    enabled: Boolean(memberId),
  });
}
