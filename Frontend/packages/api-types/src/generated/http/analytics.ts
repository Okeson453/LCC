/* eslint-disable */
/**
 * @generated
 * From schemas/openapi/api-gateway.yaml — Analytics resources.
 */

import type { DateTime } from './common';

export interface ContentAnalytics {
  impressions: number;
  engagements: number;
  followers_delta: number;
  by_pillar: Array<{ pillar: string; impressions: number; engagements: number }>;
  time_series: Array<{ date: string; impressions: number; engagements: number }>;
}

export interface ProfileAnalytics {
  profile_views: number;
  search_appearances: number;
  connection_requests_received: number;
  time_series: Array<{ date: string; views: number; searches: number }>;
}

export interface NetworkAnalytics {
  total_connections: number;
  growth_30d: number;
  warm_intro_count: number;
  stage_breakdown: Record<string, number>;
}

export interface OutreachAnalytics {
  sent_30d: number;
  reply_rate: number;
  meeting_rate: number;
  time_series: Array<{ date: string; sent: number; replies: number }>;
}

export interface FunnelStage {
  stage: string;
  count: number;
  conversion_rate: number;
}

export interface FunnelAnalytics {
  stages: FunnelStage[];
  period: '7d' | '30d' | '90d';
}

export interface AccountHealth {
  score: number;
  components: {
    quota_remaining: number;
    grounding_score: number;
    approval_throughput: number;
    recent_denial_rate: number;
  };
  quota_consumed_today: number;
  quota_cap_today: number;
  computed_at: DateTime;
  is_restricted: boolean;
  restriction_reason: string | null;
}

export interface Digest {
  period: string;
  highlights: string[];
  metrics: Record<string, unknown>;
  generated_at: DateTime;
}
