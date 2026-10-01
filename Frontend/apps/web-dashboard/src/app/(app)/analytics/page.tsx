'use client';

import { Metadata } from 'next';
import Link from 'next/link';
import { useQuery } from '@tanstack/react-query';
import { Card, CardContent, CardHeader, CardTitle, LoadingSkeleton, ErrorState } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';
import { fetchCurrentMember } from '@/lib/api/members';

export const metadata: Metadata = { title: 'Analytics' };

export default function AnalyticsHome(): React.ReactElement {
  return <Home />;
}

function Home(): React.ReactElement {
  const memberQuery = useQuery({ queryKey: ['members', 'me'] as const, queryFn: fetchCurrentMember });
  if (memberQuery.isLoading) return <LoadingSkeleton rows={4} />;
  if (memberQuery.error || !memberQuery.data) return <ErrorState onRetry={() => memberQuery.refetch()} />;
  return (
    <ComplianceGate>
      <div className="space-y-6 p-6">
        <header>
          <h1 className="text-2xl font-bold">Analytics</h1>
          <p className="text-sm text-muted-foreground">Pick a view.</p>
        </header>
        <div className="grid gap-3 md:grid-cols-3">
          {[
            ['/analytics/content', 'Content performance'],
            ['/analytics/profile', 'Profile performance'],
            ['/analytics/network', 'Network health'],
            ['/analytics/outreach', 'Outreach metrics'],
            ['/analytics/funnel/job', 'Job funnel'],
            ['/analytics/funnel/client', 'Client funnel'],
            ['/analytics/account-health', 'Account health'],
            ['/analytics/digest/weekly', 'Weekly digest'],
            ['/analytics/digest/monthly', 'Monthly digest'],
          ].map(([href, label]) => (
            <Link key={href} href={href!}>
              <Card className="hover:bg-muted/40"><CardHeader><CardTitle className="text-sm">{label}</CardTitle></CardHeader></Card>
            </Link>
          ))}
        </div>
      </div>
    </ComplianceGate>
  );
}
