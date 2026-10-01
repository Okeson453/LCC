'use client';

import * as React from 'react';
import { useQuery } from '@tanstack/react-query';
import { Card, CardContent, CardHeader, CardTitle, LoadingSkeleton, ErrorState, Tabs, TabsList, TabsTrigger, TabsContent } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';
import { fetchCurrentMember } from '@/lib/api/members';
import { getAnalytics } from '@/lib/api/analytics';

export type AnalyticsShellProps = {
  metric: 'content' | 'profile' | 'network' | 'outreach' | 'funnel/job' | 'funnel/client' | 'account-health' | 'digest/weekly' | 'digest/monthly';
  title: string;
};

export function AnalyticsShellClient({ metric, title }: AnalyticsShellProps): React.ReactElement {
  const memberQuery = useQuery({ queryKey: ['members', 'me'] as const, queryFn: fetchCurrentMember });
  if (memberQuery.isLoading) return <LoadingSkeleton rows={6} />;
  if (memberQuery.error || !memberQuery.data) return <ErrorState onRetry={() => memberQuery.refetch()} />;
  return <Body memberId={memberQuery.data.id} metric={metric} title={title} />;
}

function Body({ memberId, metric, title }: { memberId: string; metric: AnalyticsShellProps['metric']; title: string }): React.ReactElement {
  const q = useQuery({
    queryKey: ['analytics', metric, memberId] as const,
    queryFn: () => getAnalytics(memberId, metric),
  });

  return (
    <ComplianceGate>
      <div className="space-y-6 p-6">
        <header>
          <h1 className="text-2xl font-bold">{title}</h1>
        </header>
        {q.isLoading ? <LoadingSkeleton rows={6} /> :
          q.error ? <ErrorState onRetry={() => q.refetch()} /> : (
            <Card>
              <CardHeader><CardTitle className="text-base">Trend</CardTitle></CardHeader>
              <CardContent>
                <Tabs defaultValue="7d">
                  <TabsList>
                    <TabsTrigger value="7d">7d</TabsTrigger>
                    <TabsTrigger value="30d">30d</TabsTrigger>
                    <TabsTrigger value="90d">90d</TabsTrigger>
                  </TabsList>
                  <TabsContent value="7d"><div className="h-48 rounded-md border bg-muted/30" /></TabsContent>
                  <TabsContent value="30d"><div className="h-48 rounded-md border bg-muted/30" /></TabsContent>
                  <TabsContent value="90d"><div className="h-48 rounded-md border bg-muted/30" /></TabsContent>
                </Tabs>
              </CardContent>
            </Card>
          )
        }
      </div>
    </ComplianceGate>
  );
}
