'use client';

import { useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { Card, CardContent, CardDescription, CardHeader, CardTitle, LoadingSkeleton } from '@lcc/ui';
import { runProfileAudit } from '@/lib/api/profile';

export default function AuditStep(): React.ReactElement {
  const router = useRouter();

  useEffect(() => {
    let active = true;
    (async () => {
      try {
        await runProfileAudit('');
        // poll job status here in a real impl
      } catch {
        // swallow — fallback is to proceed
      } finally {
        if (active) router.push('/onboarding/done');
      }
    })();
    return () => {
      active = false;
    };
  }, [router]);

  return (
    <Card>
      <CardHeader>
        <CardTitle>Step 5: Running profile audit</CardTitle>
        <CardDescription>This usually takes 30–60 seconds…</CardDescription>
      </CardHeader>
      <CardContent>
        <LoadingSkeleton rows={6} height="h-12" />
      </CardContent>
    </Card>
  );
}
