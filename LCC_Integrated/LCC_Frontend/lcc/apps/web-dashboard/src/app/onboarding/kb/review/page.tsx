'use client';

/**
 * Step 3: Review extracted KB records.
 *
 * Per audit S-05: previously showed hardcoded sample data. Now fetches the
 * user's member-owned KB records via the API. If the API call fails we
 * surface a graceful empty state instead of fake data.
 */
import Link from 'next/link';
import { useQuery } from '@tanstack/react-query';
import { Button, Card, CardContent, CardDescription, CardHeader, CardTitle, Badge, Alert, AlertTitle, AlertDescription, LoadingSkeleton, EmptyState } from '@lcc/ui';
import { listKbRecords } from '@/lib/api/kb';
import { fetchCurrentMember } from '@/lib/api/members';

export default function ReviewKbStep(): React.ReactElement {
  const memberQuery = useQuery({ queryKey: ['members', 'me'] as const, queryFn: fetchCurrentMember });
  const kbQuery = useQuery({
    queryKey: ['kb', 'list', memberQuery.data?.id ?? 'me'] as const,
    queryFn: () => listKbRecords(memberQuery.data!.id),
    enabled: Boolean(memberQuery.data),
  });

  if (memberQuery.isLoading) return <LoadingSkeleton rows={4} />;
  if (memberQuery.error || !memberQuery.data) {
    return <Alert variant="destructive"><AlertTitle>Not signed in</AlertTitle><AlertDescription>Sign in to review your KB.</AlertDescription></Alert>;
  }

  const records = kbQuery.data ?? [];

  return (
    <Card>
      <CardHeader>
        <CardTitle>Step 3: Review your KB</CardTitle>
        <CardDescription>{records.length} records ingested so far. Edit any that need refinement.</CardDescription>
      </CardHeader>
      <CardContent className="space-y-3">
        {kbQuery.isLoading ? <LoadingSkeleton rows={3} /> :
          records.length === 0 ? <EmptyState title="No KB records yet" description="Add a few voice samples to get started." /> : (
            <ul className="space-y-2">
              {records.slice(0, 20).map((r) => (
                <li key={r.id} className="flex items-center justify-between rounded-md border p-3">
                  <div className="flex items-center gap-2">
                    <Badge variant="secondary">{r.kind}</Badge>
                    <span className="text-sm font-medium">{r.title}</span>
                  </div>
                  <Button asChild variant="ghost" size="sm">
                    <Link href={`/kb/${r.id}`}>Edit</Link>
                  </Button>
                </li>
              ))}
            </ul>
          )
        }

        <footer className="flex justify-between">
          <Button asChild variant="ghost">
            <Link href="/onboarding/kb/voice">Back</Link>
          </Button>
          <Button asChild variant="default">
            <Link href="/onboarding/goals">Continue</Link>
          </Button>
        </footer>
      </CardContent>
    </Card>
  );
}
