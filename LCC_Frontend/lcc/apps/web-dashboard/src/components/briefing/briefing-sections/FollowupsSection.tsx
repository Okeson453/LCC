'use client';

import Link from 'next/link';
import { useQuery } from '@tanstack/react-query';
import { BriefingSection } from '../BriefingSection';
import { BriefingEmptyState } from '../BriefingEmptyState';
import { listContacts } from '@/lib/api/network';
import { differenceInDays } from 'date-fns';

export interface FollowupsSectionProps {
  memberId: string;
}

export function FollowupsSection({ memberId }: FollowupsSectionProps): React.ReactElement {
  const q = useQuery({
    queryKey: ['network', 'contacts', 'stale', memberId] as const,
    queryFn: async () => {
      const { getStaleContacts } = await import('@/lib/api/network');
      return getStaleContacts(memberId);
    },
    staleTime: 5 * 60_000,
  });

  const now = new Date();

  return (
    <BriefingSection
      title="Stale follow-ups"
      description="Contacts with no recent interaction."
      count={q.data?.length ?? 0}
    >
      {q.isLoading ? (
        <div className="space-y-2">
          <div className="h-12 animate-pulse rounded-md bg-muted/40" />
        </div>
      ) : !q.data || q.data.length === 0 ? (
        <BriefingEmptyState message="No stale contacts." />
      ) : (
        q.data.slice(0, 5).map((c) => {
          const days = c.last_interaction_at
            ? differenceInDays(now, new Date(c.last_interaction_at))
            : null;
          return (
            <Link
              key={c.id}
              href={`/network/${c.id}`}
              className="flex items-center justify-between rounded-md border bg-card p-3 transition-colors hover:bg-muted/40"
            >
              <div>
                <p className="text-sm font-medium">{c.display_name}</p>
                <p className="text-xs text-muted-foreground">{c.headline}</p>
              </div>
              <span className="text-xs text-muted-foreground">
                {days !== null ? `${days}d ago` : 'never'}
              </span>
            </Link>
          );
        })
      )}
    </BriefingSection>
  );
}
