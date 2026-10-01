'use client';

import Link from 'next/link';
import { useQuery } from '@tanstack/react-query';
import { BriefingSection } from '../BriefingSection';
import { BriefingEmptyState } from '../BriefingEmptyState';
import { Badge } from '@lcc/ui';
import { listOpportunities } from '@/lib/api/opportunity';

export interface HotOpportunitiesSectionProps {
  memberId: string;
}

export function HotOpportunitiesSection({ memberId }: HotOpportunitiesSectionProps): React.ReactElement {
  const q = useQuery({
    queryKey: ['opportunity', 'list', memberId] as const,
    queryFn: () => listOpportunities(memberId),
    staleTime: 60_000,
  });

  const top = (q.data ?? []).slice(0, 5);

  return (
    <BriefingSection
      title="Hot opportunities"
      description="Top φ matches from the last 24 hours."
      count={top.length}
    >
      {q.isLoading ? (
        <div className="space-y-2">
          <div className="h-12 animate-pulse rounded-md bg-muted/40" />
          <div className="h-12 animate-pulse rounded-md bg-muted/40" />
        </div>
      ) : top.length === 0 ? (
        <BriefingEmptyState message="Run discovery to find new opportunities." />
      ) : (
        top.map((o) => (
          <Link
            key={o.id}
            href={`/opportunities/${o.id}`}
            className="flex items-center justify-between rounded-md border bg-card p-3 transition-colors hover:bg-muted/40"
          >
            <div>
              <p className="text-sm font-medium">{o.title}</p>
              <p className="text-xs text-muted-foreground">{o.company}</p>
            </div>
            <div className="flex items-center gap-2">
              <Badge variant="secondary" className="font-mono">
                φ {o.fit_score.toFixed(2)}
              </Badge>
              <Badge variant="outline">{o.action_plan}</Badge>
            </div>
          </Link>
        ))
      )}
    </BriefingSection>
  );
}
