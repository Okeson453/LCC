'use client';

import { Metadata } from 'next';
import Link from 'next/link';
import { useQuery } from '@tanstack/react-query';
import { Card, CardContent, CardHeader, CardTitle, Button, Badge, LoadingSkeleton, ErrorState } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';
import { Plus } from 'lucide-react';
import { fetchCurrentMember } from '@/lib/api/members';

export const metadata: Metadata = { title: 'Compliance versions' };

export default function ComplianceVersionsPage(): React.ReactElement {
  return <ComplianceVersions />;
}

function ComplianceVersions(): React.ReactElement {
  const q = useQuery({ queryKey: ['members', 'me'] as const, queryFn: fetchCurrentMember });
  if (q.isLoading) return <LoadingSkeleton rows={3} />;
  if (q.error) return <ErrorState onRetry={() => q.refetch()} />;
  return (
    <ComplianceGate>
      <div className="space-y-6 p-6">
        <header className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold">Compliance versions</h1>
            <p className="text-sm text-muted-foreground">Bundle rules under versioned governance.</p>
          </div>
          <Button asChild><Link href="/admin/compliance/new"><Plus className="mr-1 h-4 w-4" aria-hidden="true" />New version</Link></Button>
        </header>
        <Card>
          <CardHeader><CardTitle>Recent versions</CardTitle></CardHeader>
          <CardContent>
            <ul className="space-y-2 text-sm">
              <li><Link href="/admin/compliance/v3" className="text-primary hover:underline">v3 (active)</Link> · 2024-12-01</li>
              <li><Link href="/admin/compliance/v2" className="text-primary hover:underline">v2</Link> · 2024-09-01</li>
              <li><Link href="/admin/compliance/v1" className="text-primary hover:underline">v1</Link> · 2024-06-01</li>
            </ul>
          </CardContent>
        </Card>
        <Card>
          <CardHeader><CardTitle>Restrictions</CardTitle></CardHeader>
          <CardContent>
            <Link href="/admin/compliance/restrictions" className="text-primary hover:underline">Member restrictions →</Link>
          </CardContent>
        </Card>
      </div>
    </ComplianceGate>
  );
}
