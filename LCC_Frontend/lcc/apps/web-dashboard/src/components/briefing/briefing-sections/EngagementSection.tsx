'use client';

import Link from 'next/link';
import { useQuery } from '@tanstack/react-query';
import { BriefingSection } from '../BriefingSection';
import { BriefingEmptyState } from '../BriefingEmptyState';
import { fetchQueue } from '@/lib/api/engagement';

export interface EngagementSectionProps {
  memberId: string;
}

export function EngagementSection({ memberId }: EngagementSectionProps): React.ReactElement {
  const q = useQuery({
    queryKey: ['engagement', 'queue', memberId] as const,
    queryFn: () => fetchQueue(memberId),
    staleTime: 60_000,
  });

  return (
    <BriefingSection
      title="Engagement"
      description="Daily ritual queue — comments, congratulations, follow-ups."
      count={q.data?.length ?? 0}
    >
      {q.isLoading ? (
        <div className="space-y-2">
          <div className="h-12 animate-pulse rounded-md bg-muted/40" />
          <div className="h-12 animate-pulse rounded-md bg-muted/40" />
        </div>
      ) : !q.data || q.data.length === 0 ? (
        <BriefingEmptyState message="Nothing queued for today." />
      ) : (
        q.data.slice(0, 5).map((item) => (
          <Link
            key={item.id}
            href={`/engagement/${item.id}`}
            className="block rounded-md border bg-card p-3 transition-colors hover:bg-muted/40"
          >
            <p className="text-sm">{item.body}</p>
            <p className="mt-1 text-xs text-muted-foreground">{item.actor.display_name}</p>
          </Link>
        ))
      )}
    </BriefingSection>
  );
}
