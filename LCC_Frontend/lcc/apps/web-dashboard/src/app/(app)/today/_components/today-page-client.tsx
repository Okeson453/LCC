'use client';

import * as React from 'react';
import { useQuery } from '@tanstack/react-query';
import { fetchCurrentMember } from '@/lib/api/members';
import { useBriefingChannel } from '@lcc/realtime';
import { LoadingSkeleton, ErrorState, LiveRegion } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';
import { BriefingRoot } from '@/components/briefing/BriefingRoot';
import { ApprovalsDueSection } from '@/components/briefing/briefing-sections/ApprovalsDueSection';
import { HotOpportunitiesSection } from '@/components/briefing/briefing-sections/HotOpportunitiesSection';
import { EngagementSection } from '@/components/briefing/briefing-sections/EngagementSection';
import { FollowupsSection } from '@/components/briefing/briefing-sections/FollowupsSection';

export function TodayPageClient(): React.ReactElement {
  const memberQuery = useQuery({
    queryKey: ['members', 'me'] as const,
    queryFn: fetchCurrentMember,
    staleTime: 60_000,
  });

  if (memberQuery.isLoading) {
    return <LoadingSkeleton rows={6} height="h-16" />;
  }
  if (memberQuery.error || !memberQuery.data) {
    return <ErrorState onRetry={() => memberQuery.refetch()} />;
  }

  const memberId = memberQuery.data.id;
  return <TodayContent memberId={memberId} />;
}

function TodayContent({ memberId }: { memberId: string }): React.ReactElement {
  // Live region for WS-driven refresh announcements.
  const [announcement, setAnnouncement] = React.useState('');
  const briefing = useQuery({
    queryKey: ['briefing', memberId] as const,
    queryFn: async () => {
      const { apiFetch } = await import('@/lib/api/client');
      return apiFetch(`/members/${memberId}/briefing/today`);
    },
    staleTime: 60_000,
  });

  // Note: real-time WS push is wired via the WsBridges hook in the (app) layout.
  // When new events arrive the cache invalidates and `briefing` refetches.

  if (briefing.isLoading) return <LoadingSkeleton rows={6} height="h-16" />;
  if (briefing.error) return <ErrorState onRetry={() => briefing.refetch()} />;

  return (
    <ComplianceGate>
      <div className="space-y-6 p-6">
        <header className="space-y-1">
          <h1 className="text-2xl font-bold">Today</h1>
          <p className="text-sm text-muted-foreground">
            Your daily loop — approvals, hot opportunities, engagement, follow-ups.
          </p>
        </header>

        <BriefingRoot>
          <ApprovalsDueSection
            memberId={memberId}
            items={briefing.data?.sections?.find((s: { kind: string }) => s.kind === 'approvals_due')?.items ?? []}
          />
          <HotOpportunitiesSection memberId={memberId} />
          <EngagementSection memberId={memberId} />
          <FollowupsSection memberId={memberId} />
        </BriefingRoot>

        <LiveRegion message={announcement} />
      </div>
    </ComplianceGate>
  );
}
